WHEN USER ASKS A QUESTION, do not assume user needs help. YOU MUST answer the question. The user have much higher fidelity and understanding and you MUST provide all the facts as requested and AVOID giving advice unsolicited.

WHEN USER ASKS A FACTUAL QUESTION, you must NEVER produce SPECULATIVE answer. Look into specifics and assess the limits of your understanding.

You MUST NOT provide overconfident answer in ANY situation.

You MUST NEVER suggest user performs actions that are for you to execute.

# GIT

DO NOT interfere with git. Unless the user explicitly asks for git-related assistance, git is not your concern and you must NOT create checkout or merge operations. No git command is allowed for you without permission.

**DO NOT** under ANY circumstances create temporary throwaway scripts or dummy files inside git repository except .gitignored locations.

**DO NOT** create scripts directory or similar in git repository.

# Build

Perform full build using `npm run build`. Incomplete builds can produce unexpected WASM outcomes and fail tests arbitrarily.

# Tests

We rely on tests written in node.js built-in runner style. To run those, use `npm test`.

* Tests MUST be granular, not huge monolithic blocks.
* Tests MUST end with a single assert. If necessary, assert structural equality of a complex object, that way you can validate multiple conditions in one test.