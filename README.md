# ORAILIX website infrastructure repository

This repository contains the code to run the backend and frontend of our website. The frontend contains the pages in HTML that are served through a Rust backend allowing quick, efficient and safe services like our AI projects.

## Setup

**Pull the code**
Straightforward

**Install Rust and Cargo**:
Follow the instructions [here](https://www.rust-lang.org/tools/install) to install Rust and Cargo.

## Compilation and Execution

1. **Compile the Project**:
   ```sh
   cargo build
   ```

2. **Run the Project**:
   ```sh
   /target/debug/autoderm_infra --ip 127.0.0.1:8080 --origin local
   ```

3. **Access the Content**:
   The content should be accessible from the browser at `127.0.0.1:8080`.


## How do I publish content?

We have a news section available under /orailix.com/news/, where you will find three categories:
- publications: papers, github code, demos
- talks: our presentations, or seminars
- tech: technical demos or project announcements

0- **Create a new branch**
   ```sh
   git checkout -b new_article
   ```
and never straight git push to main :)

Under these categories you can edit our own articles following our template, but will require a minimum of HTML knowledge. You can follow these steps:

1- **Choose a category**
Decide under which section you'd like your article to be visible

2- **Create a folder with the date in YYYY-DD-MM**
You can also copy a previous folder and rename it so you can keep the template files and modify them.

3- **Edit manifest metadatafile**
Edit manifest.txt to provide meta information about your article, this includes your title, date, location of your preview picture that you can add in the same folder, and the exact page location from the category directory. You can check in other folders how this is done

4- **Edit the page**
Your article can then be edited in index.html. You'll need some HTML coding skills, but you can always ask Lucas for help. You can run the backend code to see how your pages will look like.

6- **Save your changes and open a pull request**
Commit your changes to GitHub and open a pull request where we can check that it will not break prod, and help you with some parts :)
