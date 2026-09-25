    use super::*;
    struct DeferredProvider;
    #[async_trait]
    impl PaymentProviderGateway for DeferredProvider {
        async fn charge(&self, _: ChargeRequest) -> Result<ProviderResult, ProviderError> {
            Err(ProviderError("not configured".into()))
        }
        async fn status(&self, _: &str) -> Result<ProviderResult, ProviderError> {
            Err(ProviderError("not configured".into()))
        }
    }
    #[tokio::test]
    async fn deferred_provider_never_claims_success() {
        assert!(DeferredProvider.status("anything").await.is_err());
    }
