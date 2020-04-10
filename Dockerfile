FROM quay.io/pabrahamsson/build-s2i-rust
WORKDIR /opt/app-root/src
COPY . .
RUN rustup override set nightly && \
    cargo build --release

FROM registry.access.redhat.com/ubi8/ubi-minimal
RUN mkdir -p /application/src
WORKDIR /application
COPY --from=0 /opt/app-root/src/target/release/rust-rest .
COPY --from=0 /opt/app-root/src/Cargo.toml .
COPY --from=0 /opt/app-root/src/Rocket.toml .
COPY --from=0 /opt/app-root/src/src .
USER 1001
EXPOSE 8080
CMD ["/application/rust-rest"]
