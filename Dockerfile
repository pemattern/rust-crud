FROM messense/rust-musl-cross:x86_64 as builder
ENV SQLX_OFFLINE=true
WORKDIR /rust-crud
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY  --from=builder /rust-crud/target/x86_64-unknow-linux-musl/release/rust-crud /rust-crud
ENTRYPOINT ["/rust-crud"]
EXPOSE 3000