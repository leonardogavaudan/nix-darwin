---
name: github
description: Use this skill whenever working with GitHub via gh CLI and gh api. Covers PR/issue actions, GraphQL querying, and especially PR review lifecycle management (pending drafts, submit/approve/request-changes, editing/deleting reviews, and review threads).
---

# GitHub CLI + API Playbook

## Safety Rule

- Never post, approve, request changes, or submit a review without explicit user confirmation.
- Be explicit about mode:
  1. local-only review in terminal,
  2. pending draft review on GitHub (not submitted),
  3. submitted review event (`COMMENT`, `APPROVE`, `REQUEST_CHANGES`).

## Tool Selection

- **Read/query GitHub data:** prefer `gh api graphql`.
- **Simple actions:** use high-level `gh` commands (`gh pr create`, `gh pr checkout`, etc.).
- **Review draft lifecycle (pending/edit/submit/delete):** use `gh api` (GraphQL + REST).

Reason: `gh pr review` is submit-focused and does not expose the full pending-review lifecycle.

---

## `gh pr review`: What It Can and Cannot Do

### Can

- Submit a review immediately with:
  - `--approve`
  - `--comment`
  - `--request-changes`

### Cannot

- Create a pending (draft) review.
- Submit `--comment` with an empty body.
- Submit `--request-changes` with an empty body.
- Run non-interactively without an explicit event flag.

### Verified CLI behavior

- `gh pr review <pr>` (non-interactive) →
  `--approve, --request-changes, or --comment required when not running interactively`
- `gh pr review <pr> --comment` (or `-b ''`) →
  `body cannot be blank for comment review`
- `gh pr review <pr> --request-changes` (or `-b ''`) →
  `body cannot be blank for request-changes review`

---

## PR Review API Constraints (Important)

From GitHub review endpoints behavior:

1. **Create pending review:** omit `event` on create-review call.
2. **Create review with `COMMENT` or `REQUEST_CHANGES`:** body is required.
3. **Submit pending review:** event is required (`COMMENT`, `APPROVE`, `REQUEST_CHANGES`).
4. **Delete review:** only pending reviews can be deleted.
5. **Submitted reviews cannot be deleted.**
6. A “regular comment review with absolutely no content” is not supported by normal validation paths.

---

## Canonical Workflows

### A) Immediate submit (simple)

Use when user explicitly wants direct submission.

```bash
# Approve
gh pr review 123 --approve -b "Looks good"

# Comment
gh pr review 123 --comment -b "Left feedback below"

# Request changes
gh pr review 123 --request-changes -b "Please address X and Y"
```

### B) Draft first, submit later (pending review flow)

Use when user wants to stage/edit before sending.

### 1) Fetch PR identifiers

```bash
gh api graphql -f query='query($owner:String!, $name:String!, $number:Int!){
  repository(owner:$owner,name:$name){
    pullRequest(number:$number){
      id
      headRefOid
      reviews(last:20){
        nodes { id databaseId state body author { login } submittedAt }
      }
    }
  }
}' -F owner=OWNER -F name=REPO -F number=123
```

- `id` (GraphQL Node ID) is used by GraphQL mutations.
- `databaseId` (numeric) is used by REST endpoints.

### 2) Create pending review

```bash
gh api graphql -f query='mutation($pullRequestId:ID!, $commitOID:GitObjectID!, $body:String!){
  addPullRequestReview(input:{ pullRequestId:$pullRequestId, commitOID:$commitOID, body:$body }){
    pullRequestReview { id state url }
  }
}' \
-f pullRequestId=PR_xxx \
-f commitOID=<HEAD_SHA> \
-f body="$(< review.md)"
```

Expected state: `PENDING`.

### 3) Edit pending review body (optional)

Use REST (numeric `review_id`):

```bash
gh api -X PUT repos/OWNER/REPO/pulls/123/reviews/REVIEW_ID \
  -f body="$(< review.md)"
```

### 4) Submit pending review

GraphQL path:

```bash
gh api graphql -f query='mutation($reviewId:ID!, $event:PullRequestReviewEvent!, $body:String){
  submitPullRequestReview(input:{ pullRequestReviewId:$reviewId, event:$event, body:$body }){
    pullRequestReview { id state url }
  }
}' \
-f reviewId=PRR_xxx \
-f event=COMMENT \
-f body="$(< review.md)"
```

For approval:

```bash
... -f event=APPROVE -f body="Looks good"
```

For request changes:

```bash
... -f event=REQUEST_CHANGES -f body="Please fix X"
```

### 5) Delete pending draft (if needed)

```bash
gh api graphql -f query='mutation($reviewId:ID!){
  deletePullRequestReview(input:{ pullRequestReviewId:$reviewId }){
    pullRequestReview { id }
  }
}' -f reviewId=PRR_xxx
```

If review is already submitted, this fails (expected).

### C) Fix an already submitted review body

You cannot delete submitted reviews, but you can update the review summary body:

```bash
gh api -X PUT repos/OWNER/REPO/pulls/123/reviews/REVIEW_ID \
  -f body="Updated text"
```

---

## Review Thread Operations (GraphQL)

### List review threads

```bash
gh api graphql -f query='query($owner:String!, $name:String!, $number:Int!){
  repository(owner:$owner,name:$name){
    pullRequest(number:$number){
      reviewThreads(first:50){
        nodes {
          id
          isResolved
          path
          comments(first:20){
            nodes { id body author { login } }
          }
        }
      }
    }
  }
}' -F owner=OWNER -F name=REPO -F number=123
```

### Reply to thread

```bash
gh api graphql -f query='mutation($threadId:ID!, $body:String!){
  addPullRequestReviewThreadReply(input:{ pullRequestReviewThreadId:$threadId, body:$body }){
    comment { id }
  }
}' -f threadId=PRRT_xxx -f body='Reply text'
```

### Add inline thread to a pending review

```bash
gh api graphql -f query='mutation($reviewId:ID!, $path:String!, $line:Int!, $body:String!){
  addPullRequestReviewThread(input:{ pullRequestReviewId:$reviewId, path:$path, line:$line, body:$body }){
    thread { id }
  }
}' -f reviewId=PRR_xxx -f path='src/file.ts' -F line=42 -f body='Comment'
```

---

## Query Snippets

### Search merged PRs

```bash
gh api graphql -f query='{
  search(query: "author:USERNAME is:pr is:merged", type: ISSUE, first: 10) {
    nodes {
      ... on PullRequest {
        title number mergedAt url
        repository { nameWithOwner }
      }
    }
  }
}'
```

### Search open PRs

```bash
gh api graphql -f query='{
  search(query: "author:USERNAME is:pr is:open", type: ISSUE, first: 10) {
    nodes {
      ... on PullRequest {
        title number createdAt url reviewDecision
        repository { nameWithOwner }
      }
    }
  }
}'
```

---

## Pitfalls

- Passing `"\\u200B"` sends literal text `\u200B`, not a zero-width character.
- For exact Unicode/body content, read from a file:

```bash
gh api ... -f body="$(< review.md)"
```

- REST uses numeric `review_id`; GraphQL uses node IDs (`PRR_...`). Keep both when querying.

---

## References

- `gh pr review` manual: https://cli.github.com/manual/gh_pr_review
- REST PR reviews API: https://docs.github.com/en/rest/pulls/reviews
- GraphQL mutations reference: https://docs.github.com/en/graphql/reference/mutations
