FROM rust:1.67
WORKDIR /usr/src/merkle-tree-db
COPY . .
RUN cargo test
CMD cargo test