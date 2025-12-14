use browser_core::TabIPManager;
use virtual_ip::demo_generator;

#[tokio::test]
async fn test_tab_creation_lifecycle() {
    let generator = demo_generator();
    let manager = TabIPManager::new(generator.clone());

    let tab = manager.create_tab("US").await.unwrap();
    assert_eq!(tab.virtual_ip.country_code, "US");

    let retrieved = manager.get_tab(&tab.tab_id).await.unwrap();
    assert_eq!(retrieved.tab_id, tab.tab_id);

    manager.close_tab(&tab.tab_id).await.unwrap();
    assert!(manager.get_tab(&tab.tab_id).await.is_none());
}

#[tokio::test]
async fn test_ip_rotation() {
    let generator = demo_generator();
    let manager = TabIPManager::new(generator.clone());

    let tab = manager.create_tab("US").await.unwrap();
    let original_ip = tab.virtual_ip.ip;

    let new_ip = manager.rotate_ip(&tab.tab_id, None).await.unwrap();

    assert_ne!(original_ip, new_ip.ip);
    assert_eq!(new_ip.country_code, "US");
}

#[tokio::test]
async fn test_multiple_tabs_isolation() {
    let generator = demo_generator();
    let manager = TabIPManager::new(generator.clone());

    let tab1 = manager.create_tab("US").await.unwrap();
    let tab2 = manager.create_tab("GB").await.unwrap();
    let tab3 = manager.create_tab("DE").await.unwrap();

    assert_ne!(tab1.virtual_ip.ip, tab2.virtual_ip.ip);
    assert_ne!(tab2.virtual_ip.ip, tab3.virtual_ip.ip);
    assert_eq!(tab1.virtual_ip.country_code, "US");
    assert_eq!(tab2.virtual_ip.country_code, "GB");
    assert_eq!(tab3.virtual_ip.country_code, "DE");
}
