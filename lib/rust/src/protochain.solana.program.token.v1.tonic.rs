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
        pub async fn initialise_token2022_mint(
            &mut self,
            request: impl tonic::IntoRequest<super::InitialiseToken2022MintRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InitialiseToken2022MintResponse>,
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
                "/protochain.solana.program.token.v1.Service/InitialiseToken2022Mint",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "protochain.solana.program.token.v1.Service",
                        "InitialiseToken2022Mint",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_current_min_rent_for_token2022_mint_account(
            &mut self,
            request: impl tonic::IntoRequest<
                super::GetCurrentMinRentForToken2022MintAccountRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::GetCurrentMinRentForToken2022MintAccountResponse>,
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
                "/protochain.solana.program.token.v1.Service/GetCurrentMinRentForToken2022MintAccount",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "protochain.solana.program.token.v1.Service",
                        "GetCurrentMinRentForToken2022MintAccount",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn initialise_spl_token_mint(
            &mut self,
            request: impl tonic::IntoRequest<super::InitialiseSplTokenMintRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InitialiseSplTokenMintResponse>,
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
                "/protochain.solana.program.token.v1.Service/InitialiseSPLTokenMint",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "protochain.solana.program.token.v1.Service",
                        "InitialiseSPLTokenMint",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_current_min_rent_for_spl_token_mint_account(
            &mut self,
            request: impl tonic::IntoRequest<
                super::GetCurrentMinRentForSplTokenMintAccountRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::GetCurrentMinRentForSplTokenMintAccountResponse>,
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
                "/protochain.solana.program.token.v1.Service/GetCurrentMinRentForSPLTokenMintAccount",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "protochain.solana.program.token.v1.Service",
                        "GetCurrentMinRentForSPLTokenMintAccount",
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
        pub async fn get_current_min_rent_for_holding_account(
            &mut self,
            request: impl tonic::IntoRequest<
                super::GetCurrentMinRentForHoldingAccountRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::GetCurrentMinRentForHoldingAccountResponse>,
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
                "/protochain.solana.program.token.v1.Service/GetCurrentMinRentForHoldingAccount",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "protochain.solana.program.token.v1.Service",
                        "GetCurrentMinRentForHoldingAccount",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_holding_account(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateHoldingAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateHoldingAccountResponse>,
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
                "/protochain.solana.program.token.v1.Service/CreateHoldingAccount",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "protochain.solana.program.token.v1.Service",
                        "CreateHoldingAccount",
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
    }
}
/// Generated server implementations.
pub mod service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ServiceServer.
    #[async_trait]
    pub trait Service: Send + Sync + 'static {
        async fn initialise_token2022_mint(
            &self,
            request: tonic::Request<super::InitialiseToken2022MintRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InitialiseToken2022MintResponse>,
            tonic::Status,
        >;
        async fn get_current_min_rent_for_token2022_mint_account(
            &self,
            request: tonic::Request<
                super::GetCurrentMinRentForToken2022MintAccountRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::GetCurrentMinRentForToken2022MintAccountResponse>,
            tonic::Status,
        >;
        async fn initialise_spl_token_mint(
            &self,
            request: tonic::Request<super::InitialiseSplTokenMintRequest>,
        ) -> std::result::Result<
            tonic::Response<super::InitialiseSplTokenMintResponse>,
            tonic::Status,
        >;
        async fn get_current_min_rent_for_spl_token_mint_account(
            &self,
            request: tonic::Request<
                super::GetCurrentMinRentForSplTokenMintAccountRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<super::GetCurrentMinRentForSplTokenMintAccountResponse>,
            tonic::Status,
        >;
        async fn parse_mint(
            &self,
            request: tonic::Request<super::ParseMintRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ParseMintResponse>,
            tonic::Status,
        >;
        async fn get_current_min_rent_for_holding_account(
            &self,
            request: tonic::Request<super::GetCurrentMinRentForHoldingAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::GetCurrentMinRentForHoldingAccountResponse>,
            tonic::Status,
        >;
        async fn create_holding_account(
            &self,
            request: tonic::Request<super::CreateHoldingAccountRequest>,
        ) -> std::result::Result<
            tonic::Response<super::CreateHoldingAccountResponse>,
            tonic::Status,
        >;
        async fn mint(
            &self,
            request: tonic::Request<super::MintRequest>,
        ) -> std::result::Result<tonic::Response<super::MintResponse>, tonic::Status>;
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
                "/protochain.solana.program.token.v1.Service/InitialiseToken2022Mint" => {
                    #[allow(non_camel_case_types)]
                    struct InitialiseToken2022MintSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::InitialiseToken2022MintRequest>
                    for InitialiseToken2022MintSvc<T> {
                        type Response = super::InitialiseToken2022MintResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::InitialiseToken2022MintRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::initialise_token2022_mint(&inner, request)
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
                        let method = InitialiseToken2022MintSvc(inner);
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
                "/protochain.solana.program.token.v1.Service/GetCurrentMinRentForToken2022MintAccount" => {
                    #[allow(non_camel_case_types)]
                    struct GetCurrentMinRentForToken2022MintAccountSvc<T: Service>(
                        pub Arc<T>,
                    );
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<
                        super::GetCurrentMinRentForToken2022MintAccountRequest,
                    > for GetCurrentMinRentForToken2022MintAccountSvc<T> {
                        type Response = super::GetCurrentMinRentForToken2022MintAccountResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GetCurrentMinRentForToken2022MintAccountRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::get_current_min_rent_for_token2022_mint_account(
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
                        let method = GetCurrentMinRentForToken2022MintAccountSvc(inner);
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
                "/protochain.solana.program.token.v1.Service/InitialiseSPLTokenMint" => {
                    #[allow(non_camel_case_types)]
                    struct InitialiseSPLTokenMintSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::InitialiseSplTokenMintRequest>
                    for InitialiseSPLTokenMintSvc<T> {
                        type Response = super::InitialiseSplTokenMintResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::InitialiseSplTokenMintRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::initialise_spl_token_mint(&inner, request)
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
                        let method = InitialiseSPLTokenMintSvc(inner);
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
                "/protochain.solana.program.token.v1.Service/GetCurrentMinRentForSPLTokenMintAccount" => {
                    #[allow(non_camel_case_types)]
                    struct GetCurrentMinRentForSPLTokenMintAccountSvc<T: Service>(
                        pub Arc<T>,
                    );
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<
                        super::GetCurrentMinRentForSplTokenMintAccountRequest,
                    > for GetCurrentMinRentForSPLTokenMintAccountSvc<T> {
                        type Response = super::GetCurrentMinRentForSplTokenMintAccountResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GetCurrentMinRentForSplTokenMintAccountRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::get_current_min_rent_for_spl_token_mint_account(
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
                        let method = GetCurrentMinRentForSPLTokenMintAccountSvc(inner);
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
                "/protochain.solana.program.token.v1.Service/GetCurrentMinRentForHoldingAccount" => {
                    #[allow(non_camel_case_types)]
                    struct GetCurrentMinRentForHoldingAccountSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<
                        super::GetCurrentMinRentForHoldingAccountRequest,
                    > for GetCurrentMinRentForHoldingAccountSvc<T> {
                        type Response = super::GetCurrentMinRentForHoldingAccountResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::GetCurrentMinRentForHoldingAccountRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::get_current_min_rent_for_holding_account(
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
                        let method = GetCurrentMinRentForHoldingAccountSvc(inner);
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
                "/protochain.solana.program.token.v1.Service/CreateHoldingAccount" => {
                    #[allow(non_camel_case_types)]
                    struct CreateHoldingAccountSvc<T: Service>(pub Arc<T>);
                    impl<
                        T: Service,
                    > tonic::server::UnaryService<super::CreateHoldingAccountRequest>
                    for CreateHoldingAccountSvc<T> {
                        type Response = super::CreateHoldingAccountResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateHoldingAccountRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Service>::create_holding_account(&inner, request)
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
                        let method = CreateHoldingAccountSvc(inner);
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
