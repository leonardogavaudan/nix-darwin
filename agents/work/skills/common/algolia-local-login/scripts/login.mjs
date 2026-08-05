import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

import WebSocket from 'ws';

const execFileAsync = promisify(execFile);

const CDP_URL = 'http://127.0.0.1:9222';
const EXTENSION_ORIGIN =
  'chrome-extension://aeblfdkhhhdcdjpifhhbdiojplfjncoa';
const KEYCHAIN_ACCOUNT = 'leonardo.gavaudan@algolia.com';
const KEYCHAIN_SERVICE = 'algolia-1password-unlock';
const DEFAULT_APPLICATION_ID = 'F4T6CUV2AH';
const EXTENSION_CONTEXT_URL = `${EXTENSION_ORIGIN}/inline/notification/notification.html?language=en`;
const EXTENSION_CONTEXT_TIMEOUT_MS = 5_000;

const connectCdp = async (url) => {
  const socket = new WebSocket(url);
  const pending = new Map();
  let nextId = 1;

  await new Promise((resolve, reject) => {
    socket.addEventListener('open', resolve, { once: true });
    socket.addEventListener('error', reject, { once: true });
  });

  socket.addEventListener('message', (event) => {
    const message = JSON.parse(String(event.data));
    const request = pending.get(message.id);
    if (!request) return;

    pending.delete(message.id);
    if (message.error) request.reject(new Error(message.error.message));
    else request.resolve(message.result);
  });

  return {
    close: () => socket.close(),
    send: (method, params = {}) =>
      new Promise((resolve, reject) => {
        const id = nextId++;
        pending.set(id, { resolve, reject });
        socket.send(JSON.stringify({ id, method, params }));
      }),
  };
};

const createExtensionContext = async () => {
  const version = await fetch(`${CDP_URL}/json/version`).then((response) => response.json());
  const browserCdp = await connectCdp(version.webSocketDebuggerUrl);
  let targetCdp;
  let targetId;

  try {
    ({ targetId } = await browserCdp.send('Target.createTarget', {
      url: EXTENSION_CONTEXT_URL,
      background: true,
    }));

    const startedAt = Date.now();
    let target;
    while (Date.now() - startedAt < EXTENSION_CONTEXT_TIMEOUT_MS) {
      const targets = await fetch(`${CDP_URL}/json/list`).then((response) => response.json());
      target = targets.find((candidate) => candidate.id === targetId);
      if (target?.webSocketDebuggerUrl) break;
      await new Promise((resolve) => setTimeout(resolve, 100));
    }

    if (!target?.webSocketDebuggerUrl) {
      throw new Error('1Password extension context did not become available');
    }

    targetCdp = await connectCdp(target.webSocketDebuggerUrl);

    return {
      evaluateMessage: async (message) => {
        const evaluation = await targetCdp.send('Runtime.evaluate', {
          expression: `chrome.runtime.sendMessage(${JSON.stringify(message)})`,
          awaitPromise: true,
          returnByValue: true,
        });

        if (evaluation.exceptionDetails) {
          throw new Error('1Password extension message evaluation failed');
        }

        return evaluation.result.value;
      },
      close: async () => {
        targetCdp.close();
        await browserCdp.send('Target.closeTarget', { targetId });
        browserCdp.close();
      },
    };
  } catch (error) {
    targetCdp?.close();
    if (targetId) await browserCdp.send('Target.closeTarget', { targetId });
    browserCdp.close();
    throw error;
  }
};

const isTargetOnHost = (target, host, port) => {
  if (target.type !== 'page') return false;

  try {
    const url = new URL(target.url);
    return url.hostname === host && url.port === String(port);
  } catch {
    return false;
  }
};

const waitForTarget = async (targetId) => {
  const startedAt = Date.now();
  while (Date.now() - startedAt < EXTENSION_CONTEXT_TIMEOUT_MS) {
    const targets = await fetch(`${CDP_URL}/json/list`).then((response) =>
      response.json(),
    );
    const target = targets.find((candidate) => candidate.id === targetId);
    if (target?.webSocketDebuggerUrl) return target;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }

  throw new Error('Browser target did not become available');
};

const createPageContext = async ({ host, port }) => {
  const version = await fetch(`${CDP_URL}/json/version`).then((response) =>
    response.json(),
  );
  const browserCdp = await connectCdp(version.webSocketDebuggerUrl);
  let targetCdp;
  let prepared = false;
  let newDocumentScriptIdentifier;

  try {
    const targets = await fetch(`${CDP_URL}/json/list`).then((response) =>
      response.json(),
    );
    let target = targets.find((candidate) =>
      isTargetOnHost(candidate, host, port),
    );

    if (!target) {
      const { targetId } = await browserCdp.send('Target.createTarget', {
        url: 'about:blank',
        background: true,
      });
      target = await waitForTarget(targetId);
    }

    targetCdp = await connectCdp(target.webSocketDebuggerUrl);

    return {
      activate: () =>
        browserCdp.send('Target.activateTarget', { targetId: target.id }),
      close: async () => {
        if (newDocumentScriptIdentifier) {
          try {
            await targetCdp.send('Page.removeScriptToEvaluateOnNewDocument', {
              identifier: newDocumentScriptIdentifier,
            });
          } catch {
            // The page may have closed while the login was running.
          }
        }
        targetCdp.close();
        browserCdp.close();
      },
      evaluate: async (expression) => {
        const evaluation = await targetCdp.send('Runtime.evaluate', {
          expression,
          awaitPromise: true,
          returnByValue: true,
        });
        if (evaluation.exceptionDetails) {
          throw new Error('Local page evaluation failed');
        }
        return evaluation.result.value;
      },
      isPrepared: () => prepared,
      markPrepared: (identifier) => {
        prepared = true;
        newDocumentScriptIdentifier = identifier;
      },
      send: (method, params = {}) => targetCdp.send(method, params),
    };
  } catch (error) {
    targetCdp?.close();
    browserCdp.close();
    throw error;
  }
};

const getResponseData = (response) =>
  response?.type === 'Success' ? response.data : response;

const sendExtensionMessage = (extensionContext, message) =>
  extensionContext.evaluateMessage(message);

const getKeychainPassword = async () => {
  const { stdout } = await execFileAsync(
    '/usr/bin/security',
    [
      'find-generic-password',
      '-w',
      '-a',
      KEYCHAIN_ACCOUNT,
      '-s',
      KEYCHAIN_SERVICE,
    ],
    { encoding: 'utf8' },
  );

  const password = stdout.trimEnd();
  if (!password) {
    throw new Error(`Keychain service ${KEYCHAIN_SERVICE} is empty`);
  }

  return password;
};

const assertIndependentExtensionUnlock = async (extensionContext) => {
  const configurationResponse = await sendExtensionMessage(extensionContext, {
    name: 'get-settings-configuration',
  });
  const configuration = getResponseData(configurationResponse);

  if (configuration?.useSharedLockState) {
    throw new Error(
      'Browser Pi still shares its lock state with the 1Password desktop app. Disable "Integrate this extension with the 1Password desktop app" in the Browser Pi profile before retrying',
    );
  }
};

const unlockExtension = async (extensionPage) => {
  const lockResponse = await sendExtensionMessage(extensionPage, {
    name: 'get-extension-locked',
  });
  const lockData = getResponseData(lockResponse);
  const locked =
    typeof lockData === 'boolean' ? lockData : Boolean(lockData?.locked);

  if (!locked) {
    const accountResponse = await sendExtensionMessage(extensionPage, {
      name: 'get-account-list',
    });
    const accounts = getResponseData(accountResponse);
    const accountUuid = accounts?.find((account) => !account.locked)?.uuid ?? accounts?.[0]?.uuid;

    if (!accountUuid) {
      throw new Error('1Password account metadata is unavailable');
    }

    return accountUuid;
  }

  const password = await getKeychainPassword();
  const unlockResponse = await sendExtensionMessage(extensionPage, {
    name: 'validate-account-password',
    data: { password },
  });
  const unlockData = getResponseData(unlockResponse);

  if (
    unlockResponse?.type !== 'Success' ||
    unlockData?.validationError === true
  ) {
    throw new Error('1Password extension unlock failed');
  }

  const accountUuid = unlockData?.accounts?.[0]?.uuid;
  if (!accountUuid) {
    throw new Error('1Password account metadata is unavailable after unlock');
  }

  return accountUuid;
};

const getBetaCredentials = async (extensionPage, accountUuid) => {
  const listResponse = await sendExtensionMessage(extensionPage, {
    name: 'get-item-list-entries',
    data: { options: { accountUuid, searchEverywhere: true } },
  });
  const entries = getResponseData(listResponse);
  const item = entries?.find(
    (entry) =>
      entry.templateUuid === '001' &&
      entry.urls?.some((url) =>
        String(url.url ?? url).includes('beta-dashboard.algolia.com'),
      ),
  );

  if (!item) {
    throw new Error('Beta Dashboard login item was not found in 1Password');
  }

  const detailResponse = await sendExtensionMessage(extensionPage, {
    name: 'get-core-item-details',
    data: {
      itemId: item.id,
      gatherAsyncData: false,
      revealedFieldIds: [],
    },
  });
  const detailEnvelope = getResponseData(detailResponse);
  const details = getResponseData(detailEnvelope);
  const username = details?.elements?.find((element) => element.type === 'Text')
    ?.content?.dragAction?.value?.content;
  const password = details?.elements?.find(
    (element) => element.type === 'Password',
  )?.content?.dragAction?.value?.content;

  if (!username || !password) {
    throw new Error('Beta Dashboard credential fields are unavailable');
  }

  return { username, password };
};

const submitLogin = async (pageContext, credentials) => {
  const result = await pageContext.evaluate(`(async ({ username, password }) => {
    const form = document.forms[0];
    const inputs = [...document.querySelectorAll('input')];
    const emailInput = inputs.find(
      (input) =>
        input.type === 'email' ||
        /email|login|username/i.test(
          [input.name, input.id, input.autocomplete].join(' '),
        ),
    );
    const passwordInput = inputs.find((input) => input.type === 'password');

    if (!form || !emailInput || !passwordInput) {
      return { ok: false, reason: 'login-form-not-found' };
    }

    const setValue = (input, value) => {
      const setter = Object.getOwnPropertyDescriptor(
        HTMLInputElement.prototype,
        'value',
      ).set;
      setter.call(input, value);
      input.dispatchEvent(new Event('input', { bubbles: true }));
      input.dispatchEvent(new Event('change', { bubbles: true }));
    };

    setValue(emailInput, username);
    setValue(passwordInput, password);

    await fetch(form.action, {
      method: form.method || 'POST',
      body: new FormData(form),
      credentials: 'include',
      redirect: 'manual',
    });

    return { ok: true };
  })(${JSON.stringify(credentials)})`);

  if (!result.ok) {
    throw new Error(`Local login failed: ${result.reason}`);
  }
};

const prepareLocalPage = async (pageContext) => {
  if (pageContext.isPrepared()) return;

  await pageContext.send('Network.enable');
  await pageContext.send('Network.setBlockedURLs', {
    urls: ['*://*.hotjar.com/*', '*://*.hotjar.io/*'],
  });
  const { identifier } = await pageContext.send(
    'Page.addScriptToEvaluateOnNewDocument',
    {
      source: `(() => {
  Object.defineProperty(window, 'HOTJAR_SITE_ID', {
    value: 1,
    writable: true,
    configurable: true,
  });

  if (!crypto.randomUUID) {
    crypto.randomUUID = () => {
      const bytes = crypto.getRandomValues(new Uint8Array(16));
      bytes[6] = (bytes[6] & 15) | 64;
      bytes[8] = (bytes[8] & 63) | 128;

      return [...bytes]
        .map(
          (byte, index) =>
            ([4, 6, 8, 10].includes(index) ? '-' : '') +
            byte.toString(16).padStart(2, '0'),
        )
        .join('');
    };
  }
})()`,
    },
  );
  pageContext.markPrepared(identifier);
};

const navigate = async (pageContext, url) => {
  const navigation = await pageContext.send('Page.navigate', { url });
  if (navigation.errorText) {
    throw new Error(`Navigation failed: ${navigation.errorText}`);
  }

  const startedAt = Date.now();
  while (Date.now() - startedAt < 30_000) {
    const metadata = await pageContext.evaluate(`({
      readyState: document.readyState,
      title: document.title,
      url: location.href,
    })`);
    if (metadata?.readyState !== 'loading') return metadata;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }

  throw new Error('Local page navigation timed out');
};

const navigateToLocalTarget = async ({ pageContext, baseUrl, targetPath }) => {
  await pageContext.activate();
  await prepareLocalPage(pageContext);
  await navigate(pageContext, `${baseUrl}${targetPath}`);
  await new Promise((resolve) => setTimeout(resolve, 1_500));

  return pageContext.evaluate(`({ title: document.title, url: location.href })`);
};

export const loginToAlgoliaLocal = async ({
  port,
  host = 'localhost',
  applicationId = DEFAULT_APPLICATION_ID,
  targetPath = `/apps/${applicationId}/ab-tests/create`,
  onStage,
} = {}) => {
  if (!Number.isInteger(port) || port <= 0) {
    throw new Error('A valid local frontend port is required');
  }

  let pageContext;
  let extensionContext;
  let stage = 'connecting to Browser Pi';
  const markStage = (nextStage) => {
    stage = nextStage;
    onStage?.(nextStage);
  };

  try {
    markStage(stage);
    const baseUrl = `http://${host}:${port}`;
    pageContext = await createPageContext({ host, port });

    markStage('reusing the local Dashboard session');
    const sessionMetadata = await navigateToLocalTarget({
      pageContext,
      baseUrl,
      targetPath,
    });
    if (!sessionMetadata.url.includes('/users/sign_in')) {
      return {
        ok: true,
        reusedSession: true,
        url: sessionMetadata.url,
        title: sessionMetadata.title,
      };
    }

    markStage('opening the local sign-in page');
    await navigate(pageContext, `${baseUrl}/users/sign_in`);

    markStage('creating a stable 1Password context');
    extensionContext = await createExtensionContext();
    markStage('checking the Browser Pi 1Password configuration');
    await assertIndependentExtensionUnlock(extensionContext);
    markStage('unlocking the 1Password extension');
    const accountUuid = await unlockExtension(extensionContext);
    markStage('reading the beta Dashboard login item');
    const credentials = await getBetaCredentials(extensionContext, accountUuid);
    markStage('submitting the local sign-in form');
    await submitLogin(pageContext, credentials);
    markStage('opening the authenticated local route');
    const authenticatedMetadata = await navigateToLocalTarget({
      pageContext,
      baseUrl,
      targetPath,
    });
    if (authenticatedMetadata.url.includes('/users/sign_in')) {
      throw new Error('Beta Dashboard credentials were rejected');
    }

    return {
      ok: true,
      reusedSession: false,
      url: authenticatedMetadata.url,
      title: authenticatedMetadata.title,
    };
  } catch (error) {
    throw new Error(
      `Algolia local login failed while ${stage}: ${
        error instanceof Error ? error.message : String(error)
      }`,
      { cause: error },
    );
  } finally {
    try {
      await extensionContext?.close();
    } catch {
      // The temporary target may already have closed after an extension error.
    }
    await pageContext?.close();
  }
};
