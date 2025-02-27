# Tauri Project - Getting Started

This document outlines the steps to set up the development environment and run the this project.

## Prerequisites

Before you begin, ensure you have the following installed:

*   **Node.js:** (version 16 or higher recommended) - [https://nodejs.org/](https://nodejs.org/)
*   **Rust:** (stable version) - [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
*   **Tauri CLI:**  Install globally using npm:

    ```bash
    npm install -g @tauri-apps/cli
    ```

*   **System Dependencies:** Tauri requires certain system dependencies depending on your operating system.  Refer to the official Tauri documentation for details: [https://tauri.app/v1/guides/getting-started/prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)

    *   **Windows:**  WebView2 runtime.
    *   **macOS:**  Xcode command-line tools.
    *   **Linux:**  webkit2gtk, gtk3, libappindicator

## Development Environment Setup

1.  **Clone the Repository:**

    ```bash
    git clone <repository_url>
    cd <project_directory>
    ```

2.  **Install Dependencies:**

    ```bash
    npm install  # Or yarn install, pnpm install, bun install, depending on your package manager
    ```

3.  **Install Rust Target (if necessary):**

    If you are building for a specific target architecture, you may need to add the Rust target:

    ```bash
    rustup target add <target>
    ```

    Replace `<target>` with the appropriate target triple (e.g., `x86_64-unknown-linux-gnu`).

## Running the Project

1.  **Development Mode:**

    To run the project in development mode (with hot reloading), use the following command:

    ```bash
    npm run tauri dev
    ```

    This will start the development server and open the Tauri application.  Any changes you make to the front-end code will be automatically reflected in the application.

2.  **Building for Production:**

    To build the project for production, use the following command:

    ```bash
    npm run tauri build
    ```

    This will create an optimized application bundle in the `src-tauri/target/release` directory.

## Troubleshooting

*   **Missing Dependencies:** If you encounter errors related to missing dependencies, refer to the Tauri documentation for your operating system to ensure you have all the required system libraries installed.
*   **Build Errors:** If you encounter build errors, try cleaning the project and rebuilding:

    ```bash
    rm -rf src-tauri/target
    tauri build
    ```

*   **Tauri CLI Issues:** If the `tauri` command is not found, ensure that the `@tauri-apps/cli` package is installed globally and that your `npm` or `yarn` bin directory is in your system's `PATH`.

## Further Information

*   **Tauri Documentation:** [https://tauri.app/](https://tauri.app/)
*   **Rust Documentation:** [https://www.rust-lang.org/](https://www.rust-lang.org/)
