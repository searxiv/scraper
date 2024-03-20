# SearXiv: scraper

This module is responsible for getting actual data from arXiv's API. It tries to
be simple and lightweight to run on virtual private server's where internet
speed is reasonable for the task. You can run multiple scraper instances, they
all will communicate with archivist as a master service.

## How to start scraper

> [!WARNING]
> Make sure archivist is up and available from scraper machine. Scraper depends
> on it and will be furious if it could not reach archivist.

There are 2 ways to run scraper.

1. You can clone this repository, build a docker image and then run it:

    ```sh
    docker build -t searxiv-scraper .
    docker run -e SCRAPER_ARCHIVIST_URL="http://fire:9000" searxiv-scraper
    ```

1. You can use image built by GitHub CI:

    ```sh
    docker run -e SCRAPER_ARCHIVIST_URL="http://fire:9000" ghcr.io/searxiv/scraper:latest
    ```

If you need more customization, you can use environment variables from
`.env.example`.

