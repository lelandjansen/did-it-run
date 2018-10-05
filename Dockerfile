FROM rust:latest
WORKDIR /usr/src/did-it-run
COPY . .
RUN cargo install --path did_it_run
CMD diditrun
