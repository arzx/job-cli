# Job Application Tracker

A CLI tool written in Rust to track your job applications locally.

## Installation

1.  Clone the repository or navigate to the directory.
2.  Install the binary globally:

    ```bash
    cargo install --path .
    ```

    This will verify dependencies and install the `job` command to your cargo bin path.

## Usage

The tool uses the command `job`.

### Add an Application
Add a new job using positional arguments. The date is optional and defaults to today if omitted.

**Syntax**:
`job add <COMPANY> <TITLE> <DOCS> <LOCATION> [DATE]`

**Examples**:
```bash
# With default date (Today)
job add "Google" "Software Engineer" "Resume" "Remote"

# With explicit date
job add "Apple" "Software Dev" "CV" "Cupertino" "2023-10-27"
```
*Note: Use quotes if the argument contains spaces.*

### List Applications
View all tracked applications in a table format.

```bash
job list
```

### Update Status
Update the final answer/status of a job using its ID (found in `job list`).

```bash
job update --id <ID> --answer "Interview Scheduled"
```

### Delete an Application
Remove an application permanently by ID.

```bash
job delete --id <ID>
```

### Import from CSV
Bulk import applications from a CSV file using `;` as the delimiter.
Duplicates (same Company + Title) are skipped.

```bash
job import jobs.csv
```

### Export to PDF
Generate a formatted PDF report (`jobs.pdf`) of all your applications.
Handles pagination automatically for large lists.

```bash
job export
```

## Data Storage
All data is stored in `jobs.json` in the directory where you run the command.
