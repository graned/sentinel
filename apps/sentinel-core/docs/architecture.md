# Architecture Overview

This project follows Clean Architecture principles:

## Layers

### Domain Layer

- **Entities**: Core business objects with identity
- **Custom Types**: Validated domain concepts  

### Services Layer (interactors)

Single responsability layer, each service exposes all related
functions to take on a business task.

- **Services**

### Use Case Layer

Each use case is an orchestrator to take over a business rule.
For better compilation time, it is recomended to group all use case
related activites by domain, and even tho it may be complicated.

**_Example_**

- **auth_use_cases.rs**: Contains all business related cases for a given domain

### Infrastructure Layer  

- **Repositories**: Provides a set of gateways to interact with Data persistence
- **HTTP**: Web server and API
- **External**: External service integrations

## Dependency Rule

Inner layers cannot depend on outer layers:

- Domain → Nothing
- Services → Domain  
- Use Cases → Services  
- Infrastructure → Domain
