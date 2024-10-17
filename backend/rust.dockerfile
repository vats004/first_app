# Build stage
FROM rust:1.79.0-buster as builder

WORKDIR /app

# Accept the build argument
ARG DATABASE_URL

# Make sure to use the ARG in ENV
ENV DATABASE_URL=$DATABASE_URL

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release


# Production stage
FROM debian:buster-slim

WORKDIR /usr/local/bin

COPY --from=builder /app/target/release/backend .

CMD ["./backend"]

# FROM rust:1.69-buster as builder: This line tells Docker to use the official Rust 1.69 image as the base image for our build stage. The as builder part gives this stage a name, which we'll use later.
# WORKDIR /app: This sets the working directory in the container to /app. Think of it like cd /app in a terminal.
# ARG DATABASE_URL: This line defines a build argument called DATABASE_URL. This allows us to pass a value for this argument when we build the Docker image.
# ENV DATABASE_URL=$DATABASE_URL: This sets an environment variable DATABASE_URL to the value of the build argument DATABASE_URL. This makes the value available to our application.
# COPY . .: This copies the current directory (i.e., the directory containing the Dockerfile) into the container at the current working directory (/app).
# RUN cargo build --release: This runs the cargo build command with the --release flag to build our Rust application.

# FROM debian:buster-slim: This line tells Docker to use the official Debian Buster Slim image as the base image for our production stage.
# WORKDIR /usr/local/bin: This sets the working directory in the container to /usr/local/bin.
# COPY --from=builder /app/target/release/backend .: This copies the built application from the build stage (builder) to the current working directory (/usr/local/bin) in the production stage. The --from=builder part tells Docker to use the build stage as the source.
# CMD ["./backend"]: This sets the default command to run when the container starts. In this case, it runs the backend executable.