# Agent-Coder

Agent-Coder is a Rust-based coding agent that assists programmers with code generation, refactoring, debugging, and project exploration.

## Key Features & Outcomes 🚀

1. **Agentic LLM Interaction**
   - Implements an agent to help users interact with the LLM and handle tool calling.

2. **Robust Error Handling**
   - Tool calls require user confirmation and allow explicit rejection reasons to inject feedback into the LLM.
   - Tool-call errors are also fed back to the LLM for self-correction.

3. **File-Based Context Management**
   - Stores context on the file system for cross-device migration.
   - Manages session-level history (view/delete by session or message).
   - Supports temporarily loading selected history in new conversations without actual deletion.

## Usage 📖

### Prerequisites

Make sure you have Rust and Cargo installed on your system.

### Configuration

Create a `.env` file in the project root with the following format:

```
OPENAI_API_KEY=<your-api-key>
OPENAI_BASE_URL=<your-base-url>
OPENAI_MODEL_NAME=<model-name>
```

After configuring your `.env` file, you can build Agent-Coder from the command line:

```
# Build the program
cargo run --release
```

### Command Line Interface

#### Main Interface

```
# Enter interactive shell mode
> /shell

# Load project directory
> /load

# Enter history manage mode
> /history

# Start a new query session
> /query
```

#### History Manage Mode

```
# Load an existing history file
history> /load

# Show messages of a specific conversation
history (loaded)> /display <idx>

# Delete a message from a conversation
history (loaded)> /delete <idx> <msg>
```

## Case Study 💡

- Build a blackjack game

- Debug the batch rename program

- List Error Types

- History Manage Mode

<table>
  <tr>
    <td width="50%">
      <video controls width="100%" src="https://github.com/user-attachments/assets/c0909232-6c61-4dcf-8bb0-1d1255f1dbc0">
      </video>
    </td>
    <td width="50%">
      <video controls width="100%" src="https://github.com/user-attachments/assets/88bebf6f-0aca-4fbe-8485-0c234a0d456f">
      </video>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <video controls width="100%" src="https://github.com/user-attachments/assets/d87bce5a-f7cc-4a44-b2c4-af194efe7975">
      </video>
    </td>
    <td width="50%">
      <video controls width="100%" src="https://github.com/user-attachments/assets/af2cbd2a-26bf-40b6-a49d-d4611a7ff5b8">
      </video>
    </td>
  </tr>
</table>

## License

This project is licensed under the MIT License – see the [LICENSE](https://github.com/dev-bell/Agent-Coder/blob/master/LICENSE.txt) file for details.
