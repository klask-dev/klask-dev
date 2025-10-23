# [![klask.dev](https://raw.githubusercontent.com/klask-dev/klask-dev/refs/heads/upgrade-v2.1/resources/images/klask-dev-clean-v2.svg)](https://github.com/klask-dev/klask-dev)

| Branch  | Build  | Coverage  |
|---|---|---|
| master  | [![CI/CD Pipeline](https://github.com/klask-dev/klask-dev/actions/workflows/ci.yml/badge.svg)](https://github.com/klask-dev/klask-dev/actions/workflows/ci.yml)  | [![Coverage Status](https://img.shields.io/coveralls/klask-dev/klask-dev/master.svg?style=flat-square)](https://coveralls.io/github/klask-dev/klask-dev?branch=master) |
| develop  | [![Build Status](https://img.shields.io/travis/klask-dev/klask-dev/develop.svg?style=flat-square)](https://travis-ci.org/klask-dev/klask-dev)  | [![Coverage Status](https://img.shields.io/coveralls/klask-dev/klask-dev/develop.svg?style=flat-square)](https://coveralls.io/github/klask-dev/klask-dev?branch=develop) |

#### Docker
[![Docker Stars](https://img.shields.io/docker/stars/klask/klask.dev.svg?style=flat-square)](https://hub.docker.com/r/klask/klask.dev/) [![Docker pulls](https://img.shields.io/docker/pulls/klask/klask.dev.svg?style=flat-square)](https://hub.docker.com/r/klask/klask.dev/) [![Docker build](https://img.shields.io/docker/automated/klask/klask.dev.svg?style=flat-square)](https://hub.docker.com/r/klask/klask.dev/builds/)


## What is klask.dev ?
__klask.dev__ is an open source search engine for source code. This application was generated using [JHipster](https://jhipster.github.io).

## 🦀 Modern Rust Version (rust-modernization branch)

We're actively developing a modern version using state-of-the-art technologies:

- **Backend**: Rust + Axum + Tantivy + PostgreSQL
- **Frontend**: React 18 + TypeScript + Vite + Tailwind CSS
- **Features**: JWT Authentication, Real-time Search, Git Crawling

### Quick Start (Modern Version)
```bash
# Clone and switch to modern branch
git checkout rust-modernization

# Run both frontend and backend
./start-dev.sh

# Or follow detailed guides:
# - Development setup: DEVELOPMENT.md
# - Testing guide: TESTING.md
```

**Status**: 
- ✅ Authentication System
- 🚧 Search Interface (in progress)
- 📋 Repository Management (planned)
- 🎨 Syntax Highlighting (planned)

### Live demo
http://app.klask.dev/

### How to run it ?
You can run an instance easily by pulling the docker image and execute by following :

    docker run klask/klask.dev

#### docker-compose
an example of a docker-compose.yml :

```Dockerfile
version: '2'
services:
  klask-app:
    image: klask/klask.dev:latest
    ports:
      - 8080:8080
    volumes:
      - /mnt/svn:/repo
      - ./data:/klask-data
      - ./application-docker.yml:/application-docker.yml
```

`/mnt/svn` is the path to my repositories  
`./data` is the location where elasticsearch files and database were saved.  
The optional file `application-docker.yml` can overrides all properties defined in [application.yml](/src/main/resources/config/application.yml) and [application-docker.yml](/src/main/resources/config/application-docker.yml)   


## Development
Before you can build this project, you must install and configure the following dependencies on your machine:

1. [Node.js][]: We use Node to run a development web server and build the project.
   Depending on your system, you can install Node either from source or as a pre-packaged bundle.

After installing Node, you should be able to run the following command to install development tools (like
[Bower][] and [BrowserSync][]). You will only need to run this command when dependencies change in package.json.
We use [Gulp][] as our build system. Install the Gulp command-line tool globally with:

    npm install
    npm install -g gulp
    npm install -g bower
    bower update
    bower install
    gulp


Run the following commands in two separate terminals to create a blissful development experience where your browser
auto-refreshes when files change on your hard drive.

    ./mvnw
    gulp

Bower is used to manage CSS and JavaScript dependencies used in this application. You can upgrade dependencies by
specifying a newer version in `bower.json`. You can also run `bower update` and `bower install` to manage dependencies.
Add the `-h` flag on any command to see how you can use it. For example, `bower update -h`.


## Building for production

To optimize the klask.dev client for production, run:

    ./mvnw -Pprod clean package

This will concatenate and minify CSS and JavaScript files. It will also modify `index.html` so it references
these new files.

To ensure everything worked, run:

    java -jar target/*.war --spring.profiles.active=prod

Then navigate to [http://localhost:8080](http://localhost:8080) in your browser.

## Testing

Unit tests are run by [Karma][] and written with [Jasmine][]. They're located in `src/test/javascript/` and can be run with:

    gulp test


## To run with docker in production :

Utiliser les fichiers docker-compose dans src/main/docker
    
    docker-compose -f elasticsearch.yml up -d
    docker-compose -f postgresql.yml up -d
    
    java -jar target/*.war --spring.profiles.active=prod




## Continuous Integration

To setup this project in Jenkins, use the following configuration:

* Project name: `klask.dev`
* Source Code Management
    * Git Repository: `https://github.com/klask-dev/klask-dev.git`
    * Branches to build: `*/master`
    * Additional Behaviours: `Wipe out repository & force clone`
* Build Triggers
    * Poll SCM / Schedule: `H/5 * * * *`
* Build
    * Invoke Maven / Tasks: `-Pprod clean package`
* Post-build Actions
    * Publish JUnit test result report / Test Report XMLs: `build/test-results/*.xml`

[JHipster]: https://jhipster.github.io/
[Node.js]: https://nodejs.org/
[Bower]: http://bower.io/
[Gulp]: http://gulpjs.com/
[BrowserSync]: http://www.browsersync.io/
[Karma]: http://karma-runner.github.io/
[Jasmine]: http://jasmine.github.io/2.0/introduction.html
[Protractor]: https://angular.github.io/protractor/
