import { execFile } from 'node:child_process';
import { networkInterfaces } from 'node:os';
import { promisify } from 'node:util';

import puppeteer from 'puppeteer-core';

const execFileAsync = promisify(execFile);

const CDP_URL = 'http://127.0.0.1:9222';
const EXTENSION_ORIGIN =
  'chrome-extension://aeblfdkhhhdcdjpifhhbdiojplfjncoa';
const KEYCHAIN_ACCOUNT = 'leonardo.gavaudan@algolia.com';
const KEYCHAIN_SERVICE = 'algolia-1password-unlock';
const DEFAULT_APPLICATION_ID = 'F4T6CUV2AH';

const getPrivateIpv4Address = () => {
  const address = Object.values(networkInterfaces())
    .flat()
    .find(
      (candidate) =>
        candidate?.family === 'IPv4' &&
        !candidate.internal &&
        /^(10\.|172\.(1[6-9]|2\d|3[01])\.|192\.168\.)/.test(candidate.address),
    )?.address;

  if (!address) {
    throw new Error('No private IPv4 address is available for the local frontend');
  }

  return address;
};

const getResponseData = (response) =>
  response?.type === 'Success' ? response.data : response;

const sendExtensionMessage = (extensionPage, message) =>
  extensionPage.evaluate(
    async (payload) => chrome.runtime.sendMessage(payload),
    message,
  );

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

const unlockExtension = async (extensionPage) => {
  const lockResponse = await sendExtensionMessage(extensionPage, {
    name: 'get-extension-locked',
  });
  const lockData = getResponseData(lockResponse);
  const locked =
    typeof lockData === 'boolean' ? lockData : Boolean(lockData?.locked);

  if (!locked) {
    return;
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
};

const getBetaCredentials = async (extensionPage) => {
  const listResponse = await sendExtensionMessage(extensionPage, {
    name: 'get-item-list-entries',
    data: { options: { searchEverywhere: true } },
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

const submitLogin = async (page, credentials) => {
  const result = await page.evaluate(async ({ username, password }) => {
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
  }, credentials);

  if (!result.ok) {
    throw new Error(`Local login failed: ${result.reason}`);
  }
};

const prepareLocalPage = async (page) => {
  const cdpSession = await page.createCDPSession();
  await cdpSession.send('Network.enable');
  await cdpSession.send('Network.setBlockedURLs', {
    urls: ['*://*.hotjar.com/*', '*://*.hotjar.io/*'],
  });
  await page.evaluateOnNewDocument(() => {
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
              `${[4, 6, 8, 10].includes(index) ? '-' : ''}${byte
                .toString(16)
                .padStart(2, '0')}`,
          )
          .join('');
      };
    }
  });
};

export const loginToAlgoliaLocal = async ({
  port,
  host = getPrivateIpv4Address(),
  applicationId = DEFAULT_APPLICATION_ID,
  targetPath = `/apps/${applicationId}/ab-tests/create`,
} = {}) => {
  if (!Number.isInteger(port) || port <= 0) {
    throw new Error('A valid local frontend port is required');
  }

  const browser = await puppeteer.connect({
    browserURL: CDP_URL,
    defaultViewport: null,
  });
  const pages = await browser.pages();
  const extensionPage = pages.find((page) =>
    page.url().startsWith(EXTENSION_ORIGIN),
  );

  if (!extensionPage) {
    throw new Error('Open the 1Password extension popup in Browser Pi first');
  }

  await unlockExtension(extensionPage);

  const baseUrl = `http://${host}:${port}`;
  const page =
    pages.find((candidate) => candidate.url().startsWith(baseUrl)) ??
    (await browser.newPage());

  await page.bringToFront();
  await prepareLocalPage(page);
  await page.goto(`${baseUrl}/users/sign_in`, {
    waitUntil: 'domcontentloaded',
    timeout: 30_000,
  });

  const credentials = await getBetaCredentials(extensionPage);
  await submitLogin(page, credentials);
  await page.goto(`${baseUrl}${targetPath}`, {
    waitUntil: 'domcontentloaded',
    timeout: 30_000,
  });
  await new Promise((resolve) => setTimeout(resolve, 1_500));

  if (page.url().includes('/users/sign_in')) {
    throw new Error('Beta Dashboard credentials were rejected');
  }

  return {
    ok: true,
    url: page.url(),
    title: await page.title(),
  };
};
