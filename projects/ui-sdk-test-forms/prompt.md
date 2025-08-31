# **Agent Task: Generate Implementation Plan for Solana RPC Client Wrapper**

## **Persona**

# UI SDK Test Forms

You are an expert in:
- Rust async programming and Solana blockchain development
- Solana Token 2022 program and SPL Token program
- Protocol Buffers & gRPC end-to-end development
- ProtoSol architecture patterns and code generation
- NextJs UIs with serverside functions that call a protobuf server via gRPC using the connect es generated types!! Which are serialisable!! and can be passed from front to back thus without issue!
- EXPERT IN connect-es typescrpt clients where you can pick a transport: e.g. this kind of stuff you are a pro in:
import { createClient } from "@connectrpc/connect";
import { createGrpcTransport } from "@connectrpc/connect-node";
import { ElizaService } from "./gen/eliza_connect.js";

const transport = createGrpcTransport({
  baseUrl: "http://localhost:8080",
});
const client = createClient(ElizaService, transport);

And THE GOAL of here is to extend the ui project in ./ui so that we have a nice way to construct transactions on the ui making calls to instruction construction methods and transaction compile, sign submit methodds all defined in ./lib/proto and implemented in ./api.

**CRITICAL**: This prompt is to ask you to generate a deliverable called: projects/ui-sdk-test-forms/implementation-plan.md an implementation plan. The plan must be such that when fully executed THE GOAL is met.

### Deliverable: 
**Location**: `projects/ui-sdk-test-forms/implementation-plan.md`  
**Decription**: a step-by-step, purely technical, super comprehensive implementation plan for an agent to follow to achieve THE GOAL.
**Content**: Small, incremental technical steps
- Step-by-step validation checkpoints
- Technical dependencies and build ordering
- Resource management and cleanup requirements
- Each step builds on previous steps
- No orphaned or hanging code at any stage
- Prioritize incremental progress over large changes

**IMPORTANT: workflow while making this implementation plan**:
We must build up and break down the plan in passes. NO ONE SHOTTING. i.e. build up a sold solid plan bit by bit. Then look at it and, break it down into small, iterative chunks that build on each other. Look at these chunks and then go another round to break it into small steps. Review the results and make sure that the steps are small enough to be implemented safely with strong testing, but big enough to move the implementation forward. Iterate until you feel that the steps are right sized for achieving this goal!

**FORBIDDEN CONTENT**:
- Implementation timelines, schedules, or time allocations
- Human workflow recommendations or project management advice
- References to daily/weekly/monthly patterns
- Time estimates for individual steps or overall completion

## **THE GOAL further**
extend the ui project in ./ui so that we have a nice way to construct transactions on the ui making calls to instruction construction methods and transaction compile, sign submit methodds all defined in ./lib/proto and implemented in ./api. We will have a UI is a nice dashboard from which you can call all the apis in a nice way. The apis all defined in: lib/proto/protosol/solana and implemented in: api, and e2e tested in: tests/go.

--> Back of the front layout:
- there must be some way on the backend to construct centrally all of the required gRPC clients that come from the connect-es code generation that can then be made available in every server side function definition so that we can call the backend using the node transport!

--> UI layout:
- there will be a sidebar on the left on which you get a link tree to go to all the pages in the dashboard
- each page provides a set of components that can be used to construct requests to make to services in the backend hosting the api, and then where relevant to pass the output of 1 to another when necessary. PROBABLY only necessary when calling some program service provider (e.g. program/system/v1/service) to construct a transaction instruction and then copy that into a draft transaction constructed from the UI to pass to the transaction compilation service (transaction/v1/service).
- these UI components will construct requests and pass them to the server side functions to execute the request so that it can be called with node grpc from the nextjs backend to the api server in ./api.

--> Data Flow:
Javascript objects constructed using the types generated using connect-es to type control and then calling a server-side-function (fine because these are serialisable) and then in the server side function we submit using the pre-constructed client somehow:
```
// somewhere earlier

import { createClient } from "@connectrpc/connect";
import { createGrpcTransport } from "@connectrpc/connect-node";
import { TransactionV1Service } from "@protosol-lib/protosol/solana/transaction/v1/service_pb.ts";
import { ProgramSystemV1Service } from "@protosol-lib/protosol/solana/program/system/v1/service_pb.ts";

const transport = createGrpcTransport({
  baseUrl: "http://localhost:....",
});

api = {
    transaction: TransactionV1Service = createClient(TransactionV1Service, transport),
    program: {
        system: ProgramSystemV1Service: createClient(ProgramSystemV1Service, transport)
    }
}

// then in the server side function:
compileTransactionResponse = api.transaction.compile(request)
return compileTransactionResponse;
```

--> Recommended app structure (this would also be side bar):
```
└── solana
    ├── account
    │   └── v1
    │       └── page where you can call account v1 service methods: (Scope Limiter: JUST GET for now)
                On this page at the top you can choose the method you want to call!
    ├── rpc_client
    │   └── v1
    │       └── page where you can call rpc client v1 service methods (Scope Limiter: JUST min ren call)
                On this page at the top you can choose the method you want to call!
    ├── transaction
    │   └── v1
    │       └── service.proto:
    │           └── page where you can call transaction v1 service methods
                    On this page at the top you can choose the method you want to call!
                    If you choose the compile transaction method you see a special bit at the top where you 
                    can use another drop down to choose which program you want to call, then another drop down with for which method
                    you want to build a transaction for. Then you can click: add instruction to transaction and the compile transaction state holding a blank draft transaction can get the instruction added. 
                    Then once you compile successfully you get the transaction copied into the sign method form
                    then the submit method form!!! It must all flow on this solana/transaction/v1 page!
```

## CRITICAL: before work on the goal can be complete we need to confirm
1. lib/ts sdk is getting populated and build properly so we can import the types into fellow yarn workspace member ./ui and be imported there in the package.json!! So we need to look at at least:
- run code gen
- set up the exports section correctly in lib/ts/package.json
- set up the build commands there correctly in lib/ts/package.json
- make sure we can run the build from root of repo with `yarn workspace @protosol/api build`. ONce that is done all the goods should be in ./lib/ts/dist. Then:
- in the ui/package.json we need to be importing "@protosol/api: 1.0.0"
- we need to ensure the workspace is linking all that correctly, i.e. can import protosol api types from ./lib/ts and run with them

## Research and Analysis Required
Construct yourself a comprehensive research todo list and execute it to get info you need as you build the implementation plan. It should cover:
- existing work in the ui directory analyse to understand where we are. That was a very rough almost hello world next js project that we can almost clear everything out of if necessary.
- Existing lib/proto/protosol/solana api protobuf architecture and context. PROTOBUF files are source of truth in this repo.
- Rust service implementation patterns
- Code generation workflow requirements
- look at the connect node LIVE latest documentation at: https://github.com/connectrpc/connect-es/tree/main/packages/connect-node. These are the generated types we should be getting in lib/ts/src/protosol/solana from code generation otherwise we have a MAJOR issue.