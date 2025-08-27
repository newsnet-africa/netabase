use netabase::Netabase;

#[tokio::test]
async fn scratch1() {
    let net1 = Netabase::new_test(1);
    net1.close().await;
}
#[tokio::test]
async fn scratch2() {
    let net2 = Netabase::new_test(2);
    net2.close().await;
}
