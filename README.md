# SearXiv: scraper

This module is responsible for getting actual data from arXiv's api. It tries to
be simple and lightweight to run on virtual private server's where internet
speed is reasonable. You can run multiple scraper instances, they all will
communicate with archivist as a master service.

## How to startup scraper

Make sure archivist is up and available from scraper machine. Copy provided
`.env.example` into `.env` and make sure all of the variables fit your needs.
For development environment there is no need to change anything.

To start scraper use docker:

```sh
$ docker build -t searxiv-scraper .
$ docker run searxiv-scraper
```

