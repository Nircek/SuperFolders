# `.prompts`

I wanted to be able to see my prompts and my agents and models outputs.

## Accessing and Managing Prompts (`.prompts`)

This guide explains how to view and save your interaction prompts and the resulting outputs from agents and models (like Task, Implementation Plan, and Walkthrough) into a local directory for easy review.

### Google Antigravity Agent Output Export

To easily capture the full interaction output, use the following steps:

1.  **Activate the Agent:** Ensure the agent tab is visible in the right-hand pane of the interface.
2.  **Locate the Output File:**
      * Open the desired output file in the right pane (e.g., **Task**, **Implementation Plan**, or **Walkthrough**).
3.  **Copy the Path:**
      * **Right-click** on the open file's tab/name.
      * Select **Copy relative path**.
      * *(The path will look something like: `/Users/$USER/.gemini/antigravity/brain/d27d1a5c-2555-4533-8312-63a33fcd7daa/walkthrough.md.resolved`)*
4.  **Copy Files to Local Directory:**
      * Open your terminal and use the copied path (up to the unique ID) to copy all related files into your target directory.
      * **Action:** Run the command:
        ```bash
        cp /Users/$USER/.gemini/antigravity/brain/<UNIQUE-ID>/*.md .prompts/0001-init/
        ```
      * *(**Note:** Replace `<UNIQUE-ID>` with the unique directory name from your copied path, e.g., `d27d1a5c-2555-4533-8312-63a33fcd7daa`)*
5.  **Export the User Prompt:**
      * In the right-hand pane, click the **three-dot menu (`...`)**.
      * Select **Export to `user_prompts.md`**.
