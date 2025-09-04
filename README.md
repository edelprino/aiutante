# AIUTANTE
A.I.U.T.A.N.T.E. means _Artificial Intelligence Using Text And Note To Execute_. It is an open-source project that leverages advanced AI agents to automate tasks and enhance productivity based on simple text files.

- Every agent is a Markdown (`.md`) file that contains a set of instructions and goals.
- Every agent can use tools that are simple YAML (`.yml`) files that run scripts or commands.

You can use your own agents by:

- Chat in CLI with an agent: `aiutante chat <agent-name>`
- Ask to an agent to execute a task: `aiutante run <agent-name> <task>`
- Run an agent as Telegram bot: `aiutante telegram <agent-name>` (need to set up a bot token in `TELOXIDE_BOT_TOKEN` env variable)
- Run all the agents through OpenAI compatible api for using them in all app that supporto OpenAI api: `aiutante api` (use `model` to set the name of the agent you want to use)


