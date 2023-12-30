#!/bin/bash

grpcurl -plaintext \
  -import-path ./proto \
  -proto ./proto/todos/v1/todos.proto \
  -d '{"task_id": "477991c0-b0c7-4e5e-a4dc-21a74ac6cbec"}' \
  "[::]:9090" \
  todos.v1.TodosService/DeleteTask
