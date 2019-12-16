FROM ubuntu:18.04 as temporary

RUN apt-get update -y && apt-get install -y build-essential wget pkg-config

RUN useradd -m avr

RUN mkdir /avr-test-suite-bin && chown avr:avr /avr-test-suite-bin

USER avr

RUN wget -q https://sh.rustup.rs -O /tmp/rustup.sh && sh /tmp/rustup.sh -y --profile minimal --quiet

ENV PATH=/home/avr/.cargo/bin:$PATH

COPY --chown=avr:avr . /avr-test-suite-src

USER root
RUN apt-get install -y clang
RUN apt-get install -y libelf-dev
USER avr

RUN cargo install --root /avr-test-suite-bin --path /avr-test-suite-src/src/avr-sim && cargo install --root /avr-test-suite-bin --path /avr-test-suite-src/src/avr-lit

FROM ubuntu:18.04

RUN apt-get update -y && apt-get install -y build-essential wget pkg-config gcc-avr binutils-avr avr-libc libelf-dev

RUN useradd -m avr

COPY --from=temporary --chown=avr:avr /avr-test-suite-bin/bin /avr-test-suite-bin
COPY --chown=avr:avr . /avr-test-suite-src

ENV PATH=/avr-test-suite-bin:$PATH

USER root
RUN ln -sf /avr-test-suite-src/tests /avr-tests
RUN apt-get install -y libncursesw5
RUN ln -s /lib/x86_64-linux-gnu/libncursesw.so.5 /lib/x86_64-linux-gnu/libncursesw.so.6
USER avr

WORKDIR /avr-tests

ENTRYPOINT ["avr-lit", "."]
