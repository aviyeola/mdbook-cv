Instructions
============

Use this prompt with a coding agent to generate code documentation. Use mdbook-cv to read it!

---

1. Act as a Principal Software Architect. Your task is to analyze the provided codebase/repository information and generate a comprehensive, enterprise-grade architecture documentation suite following the C4 Model format.
2. 
3. You must output the results as a series of structured files. For every file, explicitly state the target folder and file name using the exact format: [FILEPATH] documentation/c4/... [END FILEPATH]. 
4. 
5. Use the PlantUML C4-Standard library for all diagrams. Ensure you include the correct imports:
6. !include https://githubusercontent.com
7. !include https://githubusercontent.com
8. !include https://githubusercontent.com
9. 
10. =========================================
11. CRITICAL DIAGRAM PROPERTY & RELATIONSHIP INSTRUCTIONS:
12. - PROPERTIES: Every Container and Component declaration MUST contain its explicit properties/attributes using the standard C4 fields (Name, Tech/Type, Description) and, where applicable, internal Class-like fields or descriptions inside brackets to show critical API endpoints or table attributes.
13. - RELATIONSHIPS: Every relationship (`Rel`, `Rel_L`, `Rel_R`, `Rel_U`, `Rel_D`) must be explicitly declared with 3 distinct attributes: 
14.   1. The Action/Verb (e.g., "Sends order data").
15.   2. The Protocol/Transport Layer (e.g., "HTTPS/JSON", "gRPC", "AMQP").
16.   3. The Directional Flow where necessary to fix layout issues (Use explicit positional relations if the diagram gets cluttered).
17. =========================================
18. 
19. Here is the exact file structure you need to generate and save:
20. 
21. 1. [FILEPATH] documentation/c4/01_system_context.md [END FILEPATH]
22. - Executive Summary of the system architecture.
23. - Core business goals and technical constraints.
24. - Markdown table mapping all Users, Internal Systems, and External Systems.
25. - A functional description and data-ownership definition for each.
26. 
27. 2. [FILEPATH] documentation/c4/01_system_context.puml [END FILEPATH]
28. - Valid PlantUML code using C4_Context definitions (Person, System, System_Ext, Rel).
29. - Explicitly map user roles to systems and cross-system API/Webhook protocols (e.g., Rel(customer, ecommerce, "Places orders via", "HTTPS")).
30. 
31. 3. [FILEPATH] documentation/c4/02_container.md [END FILEPATH]
32. - Deep-dive explanation of the structural architectural choices.
33. - Technology stack matrix table (Container Name, Technology, Responsibility, Protocol).
34. - Detailed explanation of synchronous vs asynchronous communication boundaries.
35. 
36. 4. [FILEPATH] documentation/c4/02_container.puml [END FILEPATH]
37. - Valid PlantUML code using C4_Container definitions (Container, ContainerDb, Container_Boundary).
38. - Every container must list its tech stack (e.g., "React", "Node.js", "PostgreSQL").
39. - Relationships between containers must explicitly name the ports or protocols used (e.g., Rel(spa, api, "Fetches data", "GraphQL/HTTPS", "port 443")).
40. 
41. 5. [FILEPATH] documentation/c4/03_component.md [END FILEPATH]
42. - Deep dive into the primary/most critical backend container application architecture.
43. - Detailed breakdown of internal layers (e.g., Controllers/Routes, Services/Use Cases, Repositories/Data Access).
44. 
45. 6. [FILEPATH] documentation/c4/03_component.puml [END FILEPATH]
46. - Valid PlantUML code using C4_Component definitions (Component, ComponentDb, Container_Boundary).
47. - For each component, include internal structural properties (e.g., key public methods or endpoints handled as part of the description field).
48. - Show explicit call direction and data types passed between layers (e.g., Rel(authController, authService, "Invokes login(dto)", "In-Memory Call")).
49. 
50. 7. [FILEPATH] documentation/c4/04_code_spec.md [END FILEPATH]
51. - Code-level engineering manual (no code-level diagrams to prevent clutter).
52. - Clear mapping of primary design patterns utilized (e.g., Dependency Injection, Repository Pattern, CQRS).
53. - Core database schema entities, relationship constraints (One-to-Many, Many-to-Many), and indexing strategies.
54. - Critical code execution flows (step-by-step description of a primary transaction).
55. 
56. ADDITIONAL INSTRUCTIONS FOR GENERATION:
57. - Do not abbreviate or use placeholders like "// TODO: add more components". Write out the complete architecture.
58. - Ensure all PlantUML tags are balanced and compile perfectly without syntax errors.
59. - Do not mix PlantUML markup blocks inside markdown blocks; separate them strictly into their designated files.
60. 
61. Here is the codebase structure, configuration data, and context for analysis:
62. [PASTE YOUR REPOSITORY TREE, DOCKER-COMPOSE, CONFIG FILES, OR CORE CODE HERE]
63.