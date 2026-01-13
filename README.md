```markdown
# rstr - Rust TUI Search Tool

`rstr` is a CLI written in Rust. It allows you to search for text patterns in files using regex and provides an TUI to display the results.
```

## Usage

Start a search by providing the path and the search pattern (Regex):

```bash
rstr <path> <pattern>
```

**Example:**
Find all commented-out TODOs in configuration files:
```bash
rstr C:/configs "^#.*TODO"
```

### TUI Controls
- `q` or `Esc`: Exit the program.
