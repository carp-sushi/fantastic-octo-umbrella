#!/bin/bash

grpcurl -plaintext \
  -import-path ./proto \
  -proto ./proto/todos/v1/todos.proto \
  -d '{"name": "Tech To Learn", "owner": "github.com/carp-cobain"}' \
  "[::]:9090" \
  todos.v1.TodosService/CreateStory
