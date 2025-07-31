```mermaid
flowchart
  start
  introspect
  build-types
  build-query
  build-mutation
  create-router
  run
  start --> introspect
  introspect --> build-types
  build-types --> build-query
  build-types --> build-mutation
  build-query --> create-router
  build-mutation --> create-router
  create-router --> run
```
