FROM messense/rust-musl-cross:x86_64-musl as builder
ENV SQLX_OFFLINE=true
WORKDIR /rust-crud
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /rust-crud/target/x86_64-unknown-linux-musl/release/rust-postgres /rust-postgres
ENTRYPOINT [ "/rust-postgres" ]
EXPOSE 3000
