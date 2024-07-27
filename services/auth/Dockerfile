FROM rust:1.79

WORKDIR /usr/src/auth
COPY . .
RUN cargo build --release
CMD ["/usr/src/auth/target/release/auth"]