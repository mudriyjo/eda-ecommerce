FROM rust:1.79

WORKDIR /usr/src/catalog
COPY . .
RUN cargo build --release
CMD ["/usr/src/catalog/target/release/catalog"]