# AIUTANTE
A.I.U.T.A.N.T.E. stands for _Artificial Intelligence Using Text And Notes To Execute_.  
It is a project designed to build and run simple AI agents that automate tasks and enhance productivity, all based on Markdown files.

- Each agent is a Markdown (`.md`) file containing a set of instructions and goals.
- Each agent can use tools defined in simple YAML (`.yml`) files, which run scripts or commands.

You can use your own agents by:

- Chatting in the CLI with an agent: `aiutante chat <agent-name>`
- Asking an agent to execute a task: `aiutante run <agent-name> <task>`
- Running an agent as a Telegram bot: `aiutante telegram <agent-name>`  
  (requires setting up a bot token in the `TELOXIDE_BOT_TOKEN` environment variable)
- Accessing agents through an OpenAI-compatible server, so you can use them in any app that supports the OpenAI API:  
  `aiutante api` (use the `model` parameter to specify the name of the agent you want to use)
