# Shreddr

![Builds](https://github.com/N-Schaef/shreddr/workflows/Continuous%20integration/badge.svg)

Shreddr aims to be a lightweight electronic document management system with minimal external dependencies.

There are many good EDMS out there, like [paperless](https://github.com/the-paperless-project/paperless), [Mayan-EDMS](https://www.mayan-edms.com/), and [Ambar](https://ambar.cloud/).
If you are searching for a feature complete stable system, I'd suggest giving them a look.

That being said, Shreddr tries to be a solution for simple management of daily household documents, which do not require advanced features like invoice creation, workflows, etc.
It also tries to have minimal external dependencies.
The only external packages which are currently required are Tesseract, Imagemagick, and Unpaper for OCR-ing documents.

![Shreddr Web-Interface](shreddr.jpg?raw=true "Shreddr web interface")

## Features 
Shreddr currently supports these features:
 - Automatic (non-destroying) import of documents from a consumption directory
 - Tagging of documents according to user-specified rules
 - Basic extraction of meta-data, like dates, language, ...
 - OCR of documents without text (like scans)

It can be used either in server mode with a simple web-interface or as a CLI.

## Current State
Currently, I would consider Shreddr in a sort of alpha phase.
I created it in my spare-time to learn rust, and because I wanted a simple-to-use EDMS, which I can easily extend with more features.

Thus, the code quality and stability of the system is not yet up to production standards.
I will continuously improve the system but would not yet recommend it to be used primary EDMS, as the API and storage methods are still in flux and may lead to total data loss.

I encourage anyone anyway, to try out the system and propose features/fixes/... or even field pull requests.
I especially would welcome any help with the frontend, as I am in no means a frontend developer and struggle to provide the most basic of interfaces :sweat_smile:.

While developing I will still prioritize the features that benefit my personal use-cases the most.

### Planned features:
The following is a non-exhaustive list (in no particular order) of features I currently prioritize:
 - General performance improvements for large sets of documents
 - Improve code quality (documentation, tests, ...)
 - Optional PostgreSQL backend for storage of metadata and FTS to improve performance
 - Improved interface
 - Extraction of additional meta-data
 - Manual selection from a list of potential inferred metadata
 - Dashboard with statistics about recently added documents/tags
 - Validate API input
 - Standardize REST API
 - Support file formats other than PDF
 - Split into multiple crates with optional features

## Usage
To be able to run, Shreddr requires two directories.
The consumption directory (`-c`) and the data directory (`-d`).
Shreddr will import all documents put into the first directory and store them, together with indices, config files and logs in the data directory.

### OCR
Shreddr uses tesseract to OCR documents, which do not contain any text.
It supports multiple languages, which may be configured by the `-t` flag.

The languages have to be specified in [ISO 639](https://en.wikipedia.org/wiki/ISO_639-3) code.
The order of the languages is also the order in which tesseract will try to extract text from the documents.
For each language, the tesseract data files must be installed on the system.
You can install them in most linux distributions with the `tesseract-ocr-data-<code>` packages.

### Webserver
By default, Shreddr starts in CLI mode. 
This is only useful, if you do not want to have the program running continuously and only sporadically manage documents.
To start a webserver, you must also set the `-s` flag.
You can then access the server at `http://localhost:8000`.

Keep in mind, Shreddr does not have any user management/security, so do not expose the port to the internet.


## Installing
To install Shreddr you can either use the included `docker-compose.yml` or build it yourself using rust.
You must use rust nightly and have to install some compile-time dependencies.

For alpine the dependencies are (may be different for other distributions):
```
tesseract-ocr tesseract-ocr-dev leptonica-dev clang clang-libs llvm-dev leptonica
```
Then run:
```sh
rustup override set nightly
cargo install --path .
```

### Docker
This repository already includes a `Dockerfile` and a `docker-compose.yml` file for creating an alpine linux Shreddr web-server image.
You have to bind an external port to `8000` to access the web-interface.
Additionally, you have to mount the data and consume directories.
The consume directory should be a direct directory mount in most cases, in which you will put your PDF files.
The data directory can either be a docker volume or directory mount.

To change the supported OCR languages of the system, you also have to specify them as a comma-separated list of country codes in the `TESS_LANGUAGES` environment variable.
The docker image will automatically download the newest tesseract data files and start the Shreddr server with the correct language configuration.
