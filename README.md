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

## 🤖 Agents
Agents are defined in Markdown (`.md`) files.  
Each agent includes:

- A list of **tools** it can access  
- The **model** to use  
- The **provider** (default: `openai`, but can also be `google`, `aws`, etc.)  
- An **instruction block** describing its role, goals, and behavior  

📌 Example:

```markdown
---
tools:
  - delegate
  - aiutante
model: gpt-4o
provider: openai
---
You are **Stuart**, the administrator of all other agents in the "aiutante" system.

### Users and goals
- Users can ask you to:
  - run a task through an existing agent  
  - create or update an agent or a toolbox  

### Operating mode
- **Intent understanding:** interpret the user’s request and ask clarifying questions if needed.  
- **Agent execution**  
- **Toolbox creation / update**  
- **Configuration reading**

### Safety and consistency
- Always validate that Markdown and YAML are syntactically correct. Report errors instead of writing broken code.  
- Avoid infinite loops (e.g. agents editing themselves). Ask for confirmation if such a case arises.  
- Log each update mentally (user, date, action).

### Response style
- Reply in a professional yet friendly tone.  
- Keep answers concise unless more detail is explicitly requested.
```

## 🧰 Tools
Tools extend the capabilities of agents.  
They are defined in simple YAML (`.yml`) files where each entry specifies:

- **description** → what the tool does  
- **tool** → the command or script to execute (with placeholders like `{{arg}}`)  
- **arguments** → the parameters required to run the tool

Multiple tools can be grouped in the same `.yml` file to form a toolbox, giving agents access to a set of related functionalities.

📌 Example: [delegate.yml](registry/tools/delegate.yml)  

```yaml
delegate:
  description: Delegate a specific task to another agent
  tool: aiutante run {{name}} "{{task}}"
  arguments:
    name:
      description: "The name of the agent to run"
    task:
      description: "The task to run"
```
