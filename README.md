# Lang Anki Bot

## Getting graphql schema

    npm install -g graphql
    npm install -g apolo

    apollo schema:download --endpoint=https://learn.lingoda.com/graphql schema.json --header "Authentication: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJpYXQiOjE2NjIxNDU1NjgsImV4cCI6MTY2MjE0NzM2OCwic3ViIjoiOTkzOTIxIiwicm9sZXMiOlsiUk9MRV9TVFVERU5UIiwiUk9MRV9VU0VSIl0sInVzZXJuYW1lIjoia2V5cmlkYW5AZ21haWwuY29tIn0.JFDsKWiVQ2J_9GKKthvM3-GyP70MEG_PH2eocSEbN7KBFRmQ8sLDHkNKFFZj_uZzF-8WT1HWagclC1QmpgH6_BdKy9G3sC4alDjAztHAG74f6T68zMLmhdTYoBD3AR53Y1wu-emvSDHMDuljDGEjiY_aGrJm0Wrx_uGg0odKlC8LOmBL0odlvS2eiaZAW1-z79EqD09WTeHTJjQsL5YWl0dMAyGxtzL-OQpJrY2lAUN_bZHyevc9oxZiKW6sKJFVXtHoeIb2YhGl1K1QUkL2Ik4wdTo-UjmljQ4cNIzzqY-ODZmPEJboieb-S2-Ocz4ghy708C3-vScCQIkOBmMTzse2shtBy4hQg7rCcndLx7icO7x50ISkZUw2I1posziKzJn9hA6cKqgU_pqkXS00LzfiOLzmF9wqS1PQIrqnw56P9esGu6_7pIgRrfsC7qf17mBc-2Uc6Uu-b1_R1pfk11UPLBDbwVErI15e9-nDoR-0XxXuWGd7sYwV1C_9aF15_K9nv-dkSWfwmxFskuCYA5N6ynci9wu74-1X5-cTr7O3w54bqT1b6EP-dh6xxkjabKflozdW9HD-ERhPv_jw-WbUP7K7H1aTXcLE2oLpGQhyxCH2CbgOqxeaFSPFuDx4KVce0HInBiFe-XgtCKBCGQQ0TsxYWfxYgYeaLIk5Knw"

Convert json to graphql

https://transform.tools/json-to-graphql

## Local build

    docker buildx build \
          --progress=plain \
          --platform linux/amd64 \
          --build-arg BINARY_NAME=lang-anki-bot \
          --tag lang-anki-bot:latest \
          .
    docker run --rm -it lang-anki-bot sh

## one-arch build
    docker build \
        --build-arg BINARY_NAME=lang-anki-bot \
        --build-arg TARGETARCH=amd64 \
        --tag lang-anki-bot:latest \
        .

## manual debug

    docker run -d \
    --name rust \
    -v $(pwd)/lang-anki-bot:/app \
    rust:1.64.0 \
    tail -f /dev/null

    docker exec -it rust bash