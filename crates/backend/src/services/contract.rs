// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module contains the compile service of the backend. It receives a
//! string of Rust source code and returns the result of compiling the code.
//! For security reason we run the compilation inside a Docker container.
//! In order to ease testing, the service is parameterized by a compile
//! strategy. This allows easy mocking.

use actix_web::{
    body::BoxBody,
    rt::task::spawn_blocking,
    web::Json,
    HttpResponse,
    Responder,
};

pub use sandbox::{
    CompilationRequest,
    CompilationResult,
    FormattingRequest,
    FormattingResult,
    Sandbox,
    TestingRequest,
    TestingResult,
};

use sandbox;

// -------------------------------------------------------------------------------------------------
// TYPES
// -------------------------------------------------------------------------------------------------

pub type CompileStrategy = fn(CompilationRequest) -> sandbox::Result<CompilationResult>;

pub type TestingStrategy = fn(TestingRequest) -> sandbox::Result<TestingResult>;

pub type FormattingStrategy = fn(FormattingRequest) -> sandbox::Result<FormattingResult>;

// -------------------------------------------------------------------------------------------------
// IMPLEMENTATION
// -------------------------------------------------------------------------------------------------

/// The compile strategy that will be used in production.
/// The actual dockerized compilation will happen in here.
pub const COMPILE_SANDBOXED: CompileStrategy = |req| {
    let sandbox = Sandbox::new()?;

    sandbox.compile(&req)
};

pub async fn route_compile(
    compile_strategy: CompileStrategy,
    req: Json<CompilationRequest>,
) -> impl Responder {
    let compile_result = spawn_blocking(move || {
        compile_strategy(CompilationRequest {
            source: req.source.to_string(),
            version: req.version.to_string(),
        })
    })
    .await
    .expect("Contract compilation panicked");

    match compile_result {
        Ok(result) => {
            let compile_result = serde_json::to_string(&result).unwrap();
            HttpResponse::Ok().body(compile_result)
        }
        Err(err) => {
            eprintln!("{:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub const TEST_SANDBOXED: TestingStrategy = |req| {
    let sandbox = Sandbox::new()?;

    sandbox.test(&req)
};

pub const FORMAT_SANDBOXED: FormattingStrategy = |req| {
    let sandbox = Sandbox::new()?;

    sandbox.format(&req)
};

pub async fn route_test(
    compile_strategy: TestingStrategy,
    req: Json<TestingRequest>,
) -> impl Responder {
    let testing_result = spawn_blocking(move || {
        compile_strategy(TestingRequest {
            source: req.source.to_string(),
        })
    })
    .await
    .expect("Contract testing panicked");

    match testing_result {
        Ok(result) => {
            let testing_result = serde_json::to_string(&result).unwrap();
            HttpResponse::Ok().body(testing_result)
        }
        Err(err) => {
            eprintln!("{:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn route_format(
    formatting_strategy: FormattingStrategy,
    req: Json<FormattingRequest>,
) -> impl Responder {
    let formatting_result = spawn_blocking(move || {
        formatting_strategy(FormattingRequest {
            source: req.source.to_string(),
        })
    })
    .await
    .expect("Contract formatting panicked");

    match formatting_result {
        Ok(result) => {
            let formatting_result = serde_json::to_string(&result).unwrap();
            HttpResponse::Ok().body(formatting_result)
        }
        Err(err) => {
            eprintln!("{:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn route_status() -> HttpResponse<BoxBody> {
    HttpResponse::Ok().body("ink-compiler is live")
}

// -------------------------------------------------------------------------------------------------
// TESTS
// -------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{
        test,
        web,
        App,
    };

    /// A compile strategy mock. Accepts only `foo` as "correct" source code.
    const COMPILE_MOCKED: CompileStrategy = |req| {
        if req.source == "foo" {
            Ok(CompilationResult::Success {
                wasm: vec![],
                stdout: format!("Compilation of {} succeeded.", req.source),
                stderr: "".to_string(),
            })
        } else {
            Ok(CompilationResult::Error {
                stdout: "".to_string(),
                stderr: format!("Compilation of {} failed.", req.source),
            })
        }
    };

    /// A testing strategy mock. Accepts only `foo` as "correct" source code.
    const TESTING_MOCKED: TestingStrategy = |req| {
        if req.source == "foo" {
            Ok(TestingResult::Success {
                stdout: format!("Testing of {} succeeded.", req.source),
                stderr: "".to_string(),
            })
        } else {
            Ok(TestingResult::Error {
                stdout: "".to_string(),
                stderr: format!("Testing of {} failed.", req.source),
            })
        }
    };

    /// Simulates a compilation success on the service
    #[actix_rt::test]
    async fn test_compilation_success() {
        // TODO: Write reusable helper to setup service
        let app = test::init_service(App::new().route(
            "/",
            web::post().to(|body| route_compile(COMPILE_MOCKED, body)),
        ))
        .await;

        let req = CompilationRequest {
            source: "foo".to_string(),
            version: "4_2_0".to_string()
        };
        let req = test::TestRequest::post()
            .set_json(&req)
            .uri("/")
            .to_request();

        let res: CompilationResult = test::call_and_read_body_json(&app, req).await;

        assert_eq!(
            res,
            CompilationResult::Success {
                wasm: vec![],
                stdout: "Compilation of foo succeeded.".to_string(),
                stderr: "".to_string(),
            }
        );
    }

    /// Simulates a testing failure on the service
    #[actix_rt::test]
    async fn test_compilation_failure() {
        // TODO: Write reusable helper to setup service
        let app = test::init_service(App::new().route(
            "/",
            web::post().to(|body| route_compile(COMPILE_MOCKED, body)),
        ))
        .await;

        let req = CompilationRequest {
            source: "bar".to_string(),
            version: "4_2_0".to_string()
        };

        let req = test::TestRequest::post()
            .set_json(&req)
            .uri("/")
            .to_request();

        let res: TestingResult = test::call_and_read_body_json(&app, req).await;

        assert_eq!(
            res,
            TestingResult::Error {
                stdout: "".to_string(),
                stderr: "Compilation of bar failed.".to_string()
            }
        );
    }

    /// Simulates a testing success on the service
    #[actix_rt::test]
    async fn test_testing_success() {
        // TODO: Write reusable helper to setup service
        let app = test::init_service(
            App::new()
                .route("/", web::post().to(|body| route_test(TESTING_MOCKED, body))),
        )
        .await;

        let req = TestingRequest {
            source: "foo".to_string(),
        };
        let req = test::TestRequest::post()
            .set_json(&req)
            .uri("/")
            .to_request();

        let res: TestingResult = test::call_and_read_body_json(&app, req).await;

        assert_eq!(
            res,
            TestingResult::Success {
                stdout: "Testing of foo succeeded.".to_string(),
                stderr: "".to_string(),
            }
        );
    }

    /// Simulates a compilation failure on the service
    #[actix_rt::test]
    async fn test_testing_failure() {
        // TODO: Write reusable helper to setup service
        let app = test::init_service(
            App::new()
                .route("/", web::post().to(|body| route_test(TESTING_MOCKED, body))),
        )
        .await;

        let req = TestingRequest {
            source: "bar".to_string(),
        };
        let req = test::TestRequest::post()
            .set_json(&req)
            .uri("/")
            .to_request();

        let res: TestingResult = test::call_and_read_body_json(&app, req).await;

        assert_eq!(
            res,
            TestingResult::Error {
                stdout: "".to_string(),
                stderr: "Testing of bar failed.".to_string()
            }
        );
    }
}
