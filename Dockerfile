FROM rust:1

ARG UID=1000
ARG GID=1000
ARG USER=rust
ARG GROUP=rust
ARG DATABASE_URL

ENV RUST_BACKTRACE=1
ENV RUST_LOG=debug

RUN groupadd -g ${GID} ${GROUP} && \
    useradd -l -u ${UID} -g ${GID} ${USER}

USER ${USER}
WORKDIR /app

RUN cargo install diesel_cli --no-default-features --features postgres && \
    cargo install cargo-watch

EXPOSE 8080

COPY --chown=${USER}:${GROUP} --chmod=744 ./docker-entrypoint.sh  /usr/bin
ENTRYPOINT [ "/usr/bin/docker-entrypoint.sh" ]
CMD ["cargo", "watch", "-x", "run"]
