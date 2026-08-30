### Building and running the application

Copy `.env.example` to `.env`, replace every placeholder, then run `docker compose up --build`. Compose injects `.env` into the container at runtime; the file is excluded from Git and the Docker build context.

Never pass database, SMTP, object-store, or API credentials as Docker build arguments or bake them into the image. Production deployments must provide the same environment variables through the deployment platform's runtime secret mechanism.

The application will be available at http://localhost:30737.

### Deploying your application to the cloud

Build the image without credentials, for example: `docker build -t myapp .`.

If the deployment uses a different CPU architecture than the development machine, build for that platform: `docker build --platform=linux/amd64 -t myapp .`.

Then push it to the registry, for example: `docker push myregistry.com/myapp`.

Consult Docker's [getting started](https://docs.docker.com/go/get-started-sharing/) documentation for more detail on building and pushing.

### References

* [Docker's Rust guide](https://docs.docker.com/language/rust/)
