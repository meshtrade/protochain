// @generated
/// Generated client implementations.
pub mod service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct ServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> ServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            ServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn create_token2022_mint(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateToken2022MintRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateToken2022MintResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/protochain.solana.program.token.v1.Service/CreateToken2022Mint",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "protochain.solana.program.token.v1.Service",
                        "CreateToken2022Mint",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_spl_token_mint(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateSplTokenMintRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateSplTokenMintResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/protochain.solana.program.token.v1.Service/CreateSPLTokenMint",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "protochain.solana.program.token.v1.Service",
                        "CreateSPLTokenMint",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn parse_mint(
            &mut self,
            request: impl tonic::IntoRequest<super::ParseMintRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ParseMintResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/protochain.solana.program.token.v1.Service/ParseMint",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "protochain.solana.program.token.v1.Service",
                        "ParseMint",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_token2022_holding_account(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateToken2022HoldingAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateToken2022HoldingAccountResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/protochain.solana.program.token.v1.Service/CreateToken2022HoldingAccount",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "protochain.solana.program.token.v1.Service",
                        "CreateToken2022HoldingAccount",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_spl_token_holding_account(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateSplTokenHoldingAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateSplTokenHoldingAccountResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/protochain.solana.program.token.v1.Service/CreateSPLTokenHoldingAccount",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "protochain.solana.program.token.v1.Service",
                        "CreateSPLTokenHoldingAccount",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn mint(
            &mut self,
            request: impl tonic::IntoRequest<super::MintRequest>,
        ) -> std::result::Result<tonic::Response<super::MintResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/protochain.solana.program.token.v1.Service/Mint",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("protochain.solana.program.token.v1.Service", "Mint"),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn freeze_token_account(
            &mut self,
            request: impl tonic::IntoRequest<super::FreezeTokenAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::FreezeTokenAccountResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/protochain.solana.program.token.v1.Service/FreezeTokenAccount",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "protochain.solana.program.token.v1.Service",
                        "FreezeTokenAccount",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn thaw_token_account(
            &mut self,
            request: impl tonic::IntoRequest<super::ThawTokenAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ThawTokenAccountResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/protochain.solana.program.token.v1.Service/ThawTokenAccount",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "protochain.solana.program.token.v1.Service",
                        "ThawTokenAccount",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn close_token_account(
            &mut self,
            request: impl tonic::IntoRequest<super::CloseTokenAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CloseTokenAccountResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/protochain.solana.program.token.v1.Service/CloseTokenAccount",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "protochain.solana.program.token.v1.Service",
                        "CloseTokenAccount",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ServiceServer.
    #[async_trait]
    pub trait Service: Send + Sync + 'static {
        async fn create_token2022_mint(
            &self,
            request: tonic::Request<super::CreateToken2022MintRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateToken2022MintResponse>,
            tonic::Status,
        >;
        async fn create_spl_token_mint(
            &self,
            request: tonic::Request<super::CreateSplTokenMintRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateSplTokenMintResponse>,
            tonic::Status,
        >;
        async fn parse_mint(
            &self,
            request: tonic::Request<super::ParseMintRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ParseMintResponse>,
            tonic::Status,
        >;
        async fn create_token2022_holding_account(
            &self,
            request: tonic::Request<super::CreateToken2022HoldingAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateToken2022HoldingAccountResponse>,
            tonic::Status,
        >;
        async fn create_spl_token_holding_account(
            &self,
            request: tonic::Request<super::CreateSplTokenHoldingAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateSplTokenHoldingAccountResponse>,
            tonic::Status,
        >;
        async fn mint(
            &self,
            request: tonic::Request<super::MintRequest>,
        ) -> std::result::Result<tonic::Response<super::MintResponse>, tonic::Status>;
        async fn freeze_token_account(
            &self,
            request: tonic::Request<super::FreezeTokenAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::FreezeTokenAccountResponse>,
            tonic::Status,
        >;
        async fn thaw_token_account(
            &self,
            request: tonic::Request<super::ThawTokenAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ThawTokenAccountResponse>,
            tonic::Status,
        >;
        async fn close_token_account(
            &self,
            request: tonic::Request<super::CloseTokenAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CloseTokenAccountResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct ServiceServer<T: Service> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T: Service> ServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ServiceServer<T>
    where
        T: Service,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/protochain.solana.program.token.v1.Service/CreateToken2022Mint" => {
                    #[allow(non_camel_case_types)]
                    struct CreateToken2022MintSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::CreateToken2022MintRequest>
                    for CreateToken2022MintSvc<T> {
                        type Response = super::CreateToken2022MintResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateToken2022MintRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::create_token2022_mint(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreateToken2022MintSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/protochain.solana.program.token.v1.Service/CreateSPLTokenMint" => {
                    #[allow(non_camel_case_types)]
                    struct CreateSPLTokenMintSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::CreateSplTokenMintRequest>
                    for CreateSPLTokenMintSvc<T> {
                        type Response = super::CreateSplTokenMintResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateSplTokenMintRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::create_spl_token_mint(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreateSPLTokenMintSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/protochain.solana.program.token.v1.Service/ParseMint" => {
                    #[allow(non_camel_case_types)]
                    struct ParseMintSvc<T: Service>(pub Arc<T>);
                    impl<T: Service> tonic::server::UnaryService<super::ParseMintRequest>
                    for ParseMintSvc<T> {
                        type Response = super::ParseMintResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ParseMintRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::parse_mint(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ParseMintSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/protochain.solana.program.token.v1.Service/CreateToken2022HoldingAccount" => {
                    #[allow(non_camel_case_types)]
                    struct CreateToken2022HoldingAccountSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<
                        super::CreateToken2022HoldingAccountRequest,
                    > for CreateToken2022HoldingAccountSvc<T> {
                        type Response = super::CreateToken2022HoldingAccountResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::CreateToken2022HoldingAccountRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::create_token2022_holding_account(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreateToken2022HoldingAccountSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/protochain.solana.program.token.v1.Service/CreateSPLTokenHoldingAccount" => {
                    #[allow(non_camel_case_types)]
                    struct CreateSPLTokenHoldingAccountSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<
                        super::CreateSplTokenHoldingAccountRequest,
                    > for CreateSPLTokenHoldingAccountSvc<T> {
                        type Response = super::CreateSplTokenHoldingAccountResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::CreateSplTokenHoldingAccountRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::create_spl_token_holding_account(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreateSPLTokenHoldingAccountSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/protochain.solana.program.token.v1.Service/Mint" => {
                    #[allow(non_camel_case_types)]
                    struct MintSvc<T: Service>(pub Arc<T>);
                    impl<T: Service> tonic::server::UnaryService<super::MintRequest>
                    for MintSvc<T> {
                        type Response = super::MintResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MintRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::mint(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = MintSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/protochain.solana.program.token.v1.Service/FreezeTokenAccount" => {
                    #[allow(non_camel_case_types)]
                    struct FreezeTokenAccountSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::FreezeTokenAccountRequest>
                    for FreezeTokenAccountSvc<T> {
                        type Response = super::FreezeTokenAccountResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::FreezeTokenAccountRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::freeze_token_account(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = FreezeTokenAccountSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/protochain.solana.program.token.v1.Service/ThawTokenAccount" => {
                    #[allow(non_camel_case_types)]
                    struct ThawTokenAccountSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::ThawTokenAccountRequest>
                    for ThawTokenAccountSvc<T> {
                        type Response = super::ThawTokenAccountResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ThawTokenAccountRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::thaw_token_account(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ThawTokenAccountSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/protochain.solana.program.token.v1.Service/CloseTokenAccount" => {
                    #[allow(non_camel_case_types)]
                    struct CloseTokenAccountSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::CloseTokenAccountRequest>
                    for CloseTokenAccountSvc<T> {
                        type Response = super::CloseTokenAccountResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CloseTokenAccountRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::close_token_account(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CloseTokenAccountSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", tonic::Code::Unimplemented as i32)
                                .header(
                                    http::header::CONTENT_TYPE,
                                    tonic::metadata::GRPC_CONTENT_TYPE,
                                )
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Service> Clone for ServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: Service> tonic::server::NamedService for ServiceServer<T> {
        const NAME: &'static str = "protochain.solana.program.token.v1.Service";
    }
}
