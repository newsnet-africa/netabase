//! # Advanced Queries Example
//!
//! This example demonstrates advanced querying capabilities in Netabase including:
//! - Complex relational queries with foreign keys
//! - Custom filtering with predicates
//! - Range queries and prefix searches
//! - Batch operations for performance
//! - Analytics and aggregation queries
//! - Real-world e-commerce scenario
//!
//! Run with: `cargo run --example advanced_queries`

use std::collections::HashMap;
use std::time::Duration;

use bincode::{Decode, Encode};
use netabase_macros::{NetabaseModel, netabase_schema_module};
use netabase_store::{
    database::{NetabaseSledDatabase, NetabaseSledTree},
    traits::{NetabaseAdvancedQuery, NetabaseModel, NetabaseSecondaryKeyQuery},
};
use serde::{Deserialize, Serialize};

// Define an e-commerce schema with multiple related entities
#[netabase_schema_module(EcommerceSchema, EcommerceKeys)]
mod ecommerce_schema {
    use super::*;

    /// Customer model with personal and business information
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(CustomerKey)]
    pub struct Customer {
        #[key]
        pub id: u64,
        pub email: String,
        pub first_name: String,
        pub last_name: String,
        #[secondary_key]
        pub customer_type: CustomerType,
        #[secondary_key]
        pub country: String,
        #[secondary_key]
        pub loyalty_tier: LoyaltyTier,
        pub phone: Option<String>,
        pub date_of_birth: Option<u64>,
        pub registration_date: u64,
        pub last_login: Option<u64>,
        pub total_spent: f64,
        pub order_count: u32,
    }

    /// Product model with inventory and categorization
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(ProductKey)]
    pub struct Product {
        #[key]
        pub id: u64,
        pub name: String,
        pub description: String,
        pub sku: String,
        #[secondary_key]
        pub category_id: u64,
        #[secondary_key]
        pub brand: String,
        #[secondary_key]
        pub in_stock: bool,
        #[secondary_key]
        pub price_range: PriceRange,
        pub price: f64,
        pub cost: f64,
        pub stock_quantity: i32,
        pub weight: f64,
        pub dimensions: String,
        pub created_at: u64,
        pub updated_at: u64,
        pub rating: f64,
        pub review_count: u32,
    }

    /// Order model linking customers to their purchases
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(OrderKey)]
    pub struct Order {
        #[key]
        pub id: u64,
        #[secondary_key]
        pub customer_id: u64,
        #[secondary_key]
        pub status: OrderStatus,
        #[secondary_key]
        pub order_date: u64, // For time-based queries
        pub shipping_address: String,
        pub billing_address: String,
        pub payment_method: String,
        pub subtotal: f64,
        pub tax_amount: f64,
        pub shipping_cost: f64,
        pub total_amount: f64,
        pub discount_applied: Option<String>,
        pub tracking_number: Option<String>,
        pub shipped_date: Option<u64>,
        pub delivered_date: Option<u64>,
    }

    /// Order item model for many-to-many relationship between orders and products
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(OrderItemKey)]
    pub struct OrderItem {
        #[key]
        pub id: u64,
        #[secondary_key]
        pub order_id: u64,
        #[secondary_key]
        pub product_id: u64,
        pub quantity: u32,
        pub unit_price: f64,
        pub total_price: f64,
        pub discount_percentage: f64,
    }

    /// Category model for product organization
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(CategoryKey)]
    pub struct Category {
        #[key]
        pub id: u64,
        pub name: String,
        pub description: String,
        #[secondary_key]
        pub parent_category_id: Option<u64>,
        pub slug: String,
        pub image_url: Option<String>,
        pub is_active: bool,
        pub sort_order: u32,
    }

    /// Review model for product feedback
    #[derive(NetabaseModel, Clone, Encode, Decode, Debug, PartialEq, Serialize, Deserialize)]
    #[key_name(ReviewKey)]
    pub struct Review {
        #[key]
        pub id: u64,
        #[secondary_key]
        pub product_id: u64,
        #[secondary_key]
        pub customer_id: u64,
        #[secondary_key]
        pub rating: u8, // 1-5 stars
        pub title: String,
        pub content: String,
        pub is_verified_purchase: bool,
        pub helpful_votes: u32,
        pub created_at: u64,
    }

    // Enums for categorization and status tracking

    #[derive(
        Clone, Debug, PartialEq, Eq, Hash, Default, Encode, Decode, Serialize, Deserialize,
    )]
    pub enum CustomerType {
        #[default]
        Individual,
        Business,
        VIP,
    }

    #[derive(
        Clone, Debug, PartialEq, Eq, Hash, Default, Encode, Decode, Serialize, Deserialize,
    )]
    pub enum LoyaltyTier {
        #[default]
        Bronze,
        Silver,
        Gold,
        Platinum,
    }

    #[derive(
        Clone, Debug, PartialEq, Eq, Hash, Default, Encode, Decode, Serialize, Deserialize,
    )]
    pub enum OrderStatus {
        #[default]
        Pending,
        Processing,
        Shipped,
        Delivered,
        Cancelled,
        Refunded,
    }

    #[derive(
        Clone, Debug, PartialEq, Eq, Hash, Default, Encode, Decode, Serialize, Deserialize,
    )]
    pub enum PriceRange {
        #[default]
        Budget, // < $50
        Mid,     // $50-$200
        Premium, // $200-$500
        Luxury,  // > $500
    }
}

use ecommerce_schema::*;

/// E-commerce analytics service demonstrating advanced queries
pub struct EcommerceAnalytics {
    db: NetabaseSledDatabase<EcommerceSchema>,
    customer_tree: NetabaseSledTree<Customer, CustomerKey>,
    product_tree: NetabaseSledTree<Product, ProductKey>,
    order_tree: NetabaseSledTree<Order, OrderKey>,
    order_item_tree: NetabaseSledTree<OrderItem, OrderItemKey>,
    category_tree: NetabaseSledTree<Category, CategoryKey>,
    review_tree: NetabaseSledTree<Review, ReviewKey>,
}

impl EcommerceAnalytics {
    /// Create new analytics service
    pub fn new(db_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db = NetabaseSledDatabase::new_with_path(db_path)?;
        let customer_tree = db.get_main_tree()?;
        let product_tree = db.get_main_tree()?;
        let order_tree = db.get_main_tree()?;
        let order_item_tree = db.get_main_tree()?;
        let category_tree = db.get_main_tree()?;
        let review_tree = db.get_main_tree()?;

        Ok(Self {
            db,
            customer_tree,
            product_tree,
            order_tree,
            order_item_tree,
            category_tree,
            review_tree,
        })
    }

    // === Advanced Relational Queries ===

    /// Get complete order details with customer and product information
    pub fn get_order_details(
        &self,
        order_id: u64,
    ) -> Result<OrderDetails, Box<dyn std::error::Error>> {
        // Get the order
        let order = self
            .order_tree
            .get(OrderKey::Primary(OrderPrimaryKey(order_id)))?
            .ok_or("Order not found")?;

        // Get the customer
        let customer = self
            .customer_tree
            .get(CustomerKey::Primary(CustomerPrimaryKey(order.customer_id)))?
            .ok_or("Customer not found")?;

        // Get all order items
        let order_items = self
            .order_item_tree
            .query_by_secondary_key(OrderItemSecondaryKeys::Order_idKey(order_id))?;

        // Get product details for each item
        let mut items_with_products = Vec::new();
        for item in order_items {
            if let Some(product) = self
                .product_tree
                .get(ProductKey::Primary(ProductPrimaryKey(item.product_id)))?
            {
                items_with_products.push(OrderItemWithProduct { item, product });
            }
        }

        Ok(OrderDetails {
            order,
            customer,
            items: items_with_products,
        })
    }

    /// Find customers who bought specific products together (market basket analysis)
    pub fn find_customers_who_bought_together(
        &self,
        product_ids: &[u64],
    ) -> Result<Vec<Customer>, Box<dyn std::error::Error>> {
        let mut customer_product_map: HashMap<u64, Vec<u64>> = HashMap::new();

        // Build a map of customers to their purchased products
        for result in self.order_item_tree.iter() {
            let (_, order_item) = result?;
            customer_product_map
                .entry(order_item.order_id)
                .or_default()
                .push(order_item.product_id);
        }

        // Find orders that contain all specified products
        let mut qualifying_order_ids = Vec::new();
        for (order_id, products) in customer_product_map {
            if product_ids.iter().all(|&pid| products.contains(&pid)) {
                qualifying_order_ids.push(order_id);
            }
        }

        // Get customers for qualifying orders
        let mut customers = Vec::new();
        for order_id in qualifying_order_ids {
            if let Some(order) = self
                .order_tree
                .get(OrderKey::Primary(OrderPrimaryKey(order_id)))?
            {
                if let Some(customer) = self
                    .customer_tree
                    .get(CustomerKey::Primary(CustomerPrimaryKey(order.customer_id)))?
                {
                    customers.push(customer);
                }
            }
        }

        // Remove duplicates
        customers.sort_by_key(|c| c.id);
        customers.dedup_by_key(|c| c.id);
        Ok(customers)
    }

    /// Get customer lifetime value analysis
    pub fn analyze_customer_lifetime_value(
        &self,
    ) -> Result<Vec<CustomerLTV>, Box<dyn std::error::Error>> {
        let mut customer_stats: HashMap<u64, CustomerLTVData> = HashMap::new();

        // Collect order data per customer
        for result in self.order_tree.iter() {
            let (_, order) = result?;
            let stats = customer_stats.entry(order.customer_id).or_default();
            stats.total_spent += order.total_amount;
            stats.order_count += 1;

            if stats.first_order_date == 0 || order.order_date < stats.first_order_date {
                stats.first_order_date = order.order_date;
            }
            if order.order_date > stats.last_order_date {
                stats.last_order_date = order.order_date;
            }
        }

        // Convert to LTV analysis with customer details
        let mut results = Vec::new();
        for (customer_id, stats) in customer_stats {
            if let Some(customer) = self
                .customer_tree
                .get(CustomerKey::Primary(CustomerPrimaryKey(customer_id)))?
            {
                let days_active = if stats.last_order_date > stats.first_order_date {
                    ((stats.last_order_date - stats.first_order_date) / 86400) as f64
                } else {
                    1.0
                };

                let avg_order_value = stats.total_spent / stats.order_count as f64;
                let order_frequency = stats.order_count as f64 / (days_active / 365.0).max(1.0);
                let predicted_ltv = avg_order_value * order_frequency * 2.0; // 2-year projection

                results.push(CustomerLTV {
                    customer,
                    total_spent: stats.total_spent,
                    order_count: stats.order_count,
                    avg_order_value,
                    order_frequency,
                    predicted_ltv,
                    days_active: days_active as u32,
                });
            }
        }

        results.sort_by(|a, b| b.predicted_ltv.partial_cmp(&a.predicted_ltv).unwrap());
        Ok(results)
    }

    // === Advanced Filtering Queries ===

    /// Find products with complex criteria
    pub fn find_products_advanced(
        &self,
        criteria: ProductSearchCriteria,
    ) -> Result<Vec<Product>, Box<dyn std::error::Error>> {
        let results = self.product_tree.query_with_filter(|product| {
            // Price range filter
            if let Some(min_price) = criteria.min_price {
                if product.price < min_price {
                    return false;
                }
            }
            if let Some(max_price) = criteria.max_price {
                if product.price > max_price {
                    return false;
                }
            }

            // Rating filter
            if let Some(min_rating) = criteria.min_rating {
                if product.rating < min_rating {
                    return false;
                }
            }

            // Stock filter
            if criteria.in_stock_only && !product.in_stock {
                return false;
            }

            // Minimum reviews filter
            if let Some(min_reviews) = criteria.min_review_count {
                if product.review_count < min_reviews {
                    return false;
                }
            }

            // Brand filter
            if let Some(ref brand) = criteria.brand {
                if &product.brand != brand {
                    return false;
                }
            }

            true
        })?;

        Ok(results.into_iter().map(|(_, product)| product).collect())
    }

    /// Find high-value customers with specific behavior patterns
    pub fn find_vip_customers(&self) -> Result<Vec<Customer>, Box<dyn std::error::Error>> {
        let results = self.customer_tree.query_with_filter(|customer| {
            // High spending customers
            customer.total_spent > 5000.0
                && customer.order_count > 10
                && matches!(
                    customer.loyalty_tier,
                    LoyaltyTier::Gold | LoyaltyTier::Platinum
                )
        })?;

        Ok(results.into_iter().map(|(_, customer)| customer).collect())
    }

    // === Time-based Range Queries ===

    /// Get orders within a date range
    pub fn get_orders_in_date_range(
        &self,
        start_date: u64,
        end_date: u64,
    ) -> Result<Vec<Order>, Box<dyn std::error::Error>> {
        let results = self.order_tree.query_with_filter(|order| {
            order.order_date >= start_date && order.order_date <= end_date
        })?;

        Ok(results.into_iter().map(|(_, order)| order).collect())
    }

    /// Get revenue by time period
    pub fn get_revenue_analytics(
        &self,
        start_date: u64,
        end_date: u64,
    ) -> Result<RevenueAnalytics, Box<dyn std::error::Error>> {
        let orders = self.get_orders_in_date_range(start_date, end_date)?;

        let total_revenue: f64 = orders.iter().map(|o| o.total_amount).sum();
        let total_orders = orders.len();
        let avg_order_value = if total_orders > 0 {
            total_revenue / total_orders as f64
        } else {
            0.0
        };

        // Count by status
        let mut status_counts = HashMap::new();
        for order in &orders {
            *status_counts.entry(order.status.clone()).or_insert(0) += 1;
        }

        Ok(RevenueAnalytics {
            total_revenue,
            total_orders,
            avg_order_value,
            status_breakdown: status_counts,
            period_start: start_date,
            period_end: end_date,
        })
    }

    // === Batch Operations and Performance ===

    /// Batch update product prices with percentage increase
    pub fn batch_update_prices(
        &self,
        category_id: u64,
        percentage_increase: f64,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        // Find all products in the category
        let products = self
            .product_tree
            .query_by_secondary_key(ProductSecondaryKeys::Category_idKey(category_id))?;

        // Prepare batch updates
        let mut updates = Vec::new();
        for mut product in products {
            product.price *= 1.0 + (percentage_increase / 100.0);
            product.updated_at = chrono::Utc::now().timestamp() as u64;

            // Update price range if needed
            product.price_range = if product.price < 50.0 {
                PriceRange::Budget
            } else if product.price < 200.0 {
                PriceRange::Mid
            } else if product.price < 500.0 {
                PriceRange::Premium
            } else {
                PriceRange::Luxury
            };

            updates.push((product.key(), product));
        }

        let count = updates.len();
        self.product_tree.batch_insert_with_indexing(updates)?;
        Ok(count)
    }

    // === Aggregation and Analytics ===

    /// Generate comprehensive sales report
    pub fn generate_sales_report(&self) -> Result<SalesReport, Box<dyn std::error::Error>> {
        let mut report = SalesReport::default();

        // Overall statistics
        report.total_customers = self.customer_tree.len();
        report.total_products = self.product_tree.len();
        report.total_orders = self.order_tree.len();

        // Revenue by loyalty tier
        let mut tier_revenue: HashMap<LoyaltyTier, f64> = HashMap::new();
        for result in self.customer_tree.iter() {
            let (_, customer) = result?;
            *tier_revenue
                .entry(customer.loyalty_tier.clone())
                .or_insert(0.0) += customer.total_spent;
        }
        report.revenue_by_tier = tier_revenue;

        // Top-selling products
        let mut product_sales: HashMap<u64, (String, u32, f64)> = HashMap::new();
        for result in self.order_item_tree.iter() {
            let (_, item) = result?;
            let entry = product_sales.entry(item.product_id).or_insert_with(|| {
                if let Ok(Some(product)) = self
                    .product_tree
                    .get(ProductKey::Primary(ProductPrimaryKey(item.product_id)))
                {
                    (product.name, 0, 0.0)
                } else {
                    ("Unknown Product".to_string(), 0, 0.0)
                }
            });
            entry.1 += item.quantity;
            entry.2 += item.total_price;
        }

        let mut top_products: Vec<_> = product_sales
            .into_iter()
            .map(|(id, (name, qty, revenue))| TopProduct {
                id,
                name,
                quantity_sold: qty,
                revenue,
            })
            .collect();
        top_products.sort_by(|a, b| b.revenue.partial_cmp(&a.revenue).unwrap());
        top_products.truncate(10);
        report.top_products = top_products;

        // Order status distribution
        let mut status_counts = HashMap::new();
        for result in self.order_tree.iter() {
            let (_, order) = result?;
            *status_counts.entry(order.status.clone()).or_insert(0) += 1;
        }
        report.order_status_distribution = status_counts;

        Ok(report)
    }

    /// Create sample e-commerce data
    pub fn create_sample_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Creating sample e-commerce data...");

        // Create categories
        let categories = vec![
            Category {
                id: 1,
                name: "Electronics".to_string(),
                description: "Electronic devices and gadgets".to_string(),
                parent_category_id: None,
                slug: "electronics".to_string(),
                image_url: None,
                is_active: true,
                sort_order: 1,
            },
            Category {
                id: 2,
                name: "Clothing".to_string(),
                description: "Fashion and apparel".to_string(),
                parent_category_id: None,
                slug: "clothing".to_string(),
                image_url: None,
                is_active: true,
                sort_order: 2,
            },
            Category {
                id: 3,
                name: "Books".to_string(),
                description: "Books and literature".to_string(),
                parent_category_id: None,
                slug: "books".to_string(),
                image_url: None,
                is_active: true,
                sort_order: 3,
            },
        ];

        for category in categories {
            self.category_tree.insert(category.key(), category)?;
        }

        // Create customers
        let customers = vec![
            Customer {
                id: 1,
                email: "alice@example.com".to_string(),
                first_name: "Alice".to_string(),
                last_name: "Johnson".to_string(),
                customer_type: CustomerType::Individual,
                country: "US".to_string(),
                loyalty_tier: LoyaltyTier::Gold,
                phone: Some("+1234567890".to_string()),
                date_of_birth: Some(631152000),
                registration_date: 1640995200,
                last_login: Some(1672531200),
                total_spent: 2500.00,
                order_count: 15,
            },
            Customer {
                id: 2,
                email: "bob@company.com".to_string(),
                first_name: "Bob".to_string(),
                last_name: "Smith".to_string(),
                customer_type: CustomerType::Business,
                country: "US".to_string(),
                loyalty_tier: LoyaltyTier::Platinum,
                phone: Some("+1234567891".to_string()),
                date_of_birth: Some(599616000),
                registration_date: 1640995200,
                last_login: Some(1672531200),
                total_spent: 15000.00,
                order_count: 25,
            },
            Customer {
                id: 3,
                email: "carol@example.com".to_string(),
                first_name: "Carol".to_string(),
                last_name: "Davis".to_string(),
                customer_type: CustomerType::Individual,
                country: "CA".to_string(),
                loyalty_tier: LoyaltyTier::Silver,
                phone: None,
                date_of_birth: None,
                registration_date: 1641081600,
                last_login: Some(1672444800),
                total_spent: 850.00,
                order_count: 8,
            },
        ];

        for customer in customers {
            self.customer_tree.insert(customer.key(), customer)?;
        }

        // Create products
        let products = vec![
            Product {
                id: 1,
                name: "Gaming Laptop".to_string(),
                description: "High-performance gaming laptop".to_string(),
                sku: "LAPTOP001".to_string(),
                category_id: 1,
                brand: "TechBrand".to_string(),
                in_stock: true,
                price_range: PriceRange::Premium,
                price: 1499.99,
                cost: 1200.00,
                stock_quantity: 15,
                weight: 2.5,
                dimensions: "35x25x2 cm".to_string(),
                created_at: 1640995200,
                updated_at: 1640995200,
                rating: 4.5,
                review_count: 128,
            },
            Product {
                id: 2,
                name: "Wireless Headphones".to_string(),
                description: "Premium noise-cancelling headphones".to_string(),
                sku: "AUDIO001".to_string(),
                category_id: 1,
                brand: "AudioTech".to_string(),
                in_stock: true,
                price_range: PriceRange::Mid,
                price: 199.99,
                cost: 120.00,
                stock_quantity: 45,
                weight: 0.3,
                dimensions: "20x18x8 cm".to_string(),
                created_at: 1640995200,
                updated_at: 1640995200,
                rating: 4.7,
                review_count: 256,
            },
            Product {
                id: 3,
                name: "Cotton T-Shirt".to_string(),
                description: "Comfortable 100% cotton t-shirt".to_string(),
                sku: "SHIRT001".to_string(),
                category_id: 2,
                brand: "FashionCo".to_string(),
                in_stock: true,
                price_range: PriceRange::Budget,
                price: 24.99,
                cost: 12.00,
                stock_quantity: 100,
                weight: 0.2,
                dimensions: "Standard sizes".to_string(),
                created_at: 1640995200,
                updated_at: 1640995200,
                rating: 4.2,
                review_count: 89,
            },
            Product {
                id: 4,
                name: "Programming Book".to_string(),
                description: "Learn advanced programming concepts".to_string(),
                sku: "BOOK001".to_string(),
                category_id: 3,
                brand: "TechPublisher".to_string(),
                in_stock: false,
                price_range: PriceRange::Budget,
                price: 49.99,
                cost: 25.00,
                stock_quantity: 0,
                weight: 0.8,
                dimensions: "23x15x3 cm".to_string(),
                created_at: 1640995200,
                updated_at: 1640995200,
                rating: 4.8,
                review_count: 45,
            },
        ];

        for product in products {
            self.product_tree.insert(product.key(), product)?;
        }

        // Create orders
        let orders = vec![
            Order {
                id: 1,
                customer_id: 1,
                status: OrderStatus::Delivered,
                order_date: 1641168000,
                shipping_address: "123 Main St, City, US".to_string(),
                billing_address: "123 Main St, City, US".to_string(),
                payment_method: "Credit Card".to_string(),
                subtotal: 1724.98,
                tax_amount: 137.99,
                shipping_cost: 15.00,
                total_amount: 1877.97,
                discount_applied: Some("SAVE10".to_string()),
                tracking_number: Some("TRACK123".to_string()),
                shipped_date: Some(1641254400),
                delivered_date: Some(1641340800),
            },
            Order {
                id: 2,
                customer_id: 2,
                status: OrderStatus::Processing,
                order_date: 1641254400,
                shipping_address: "456 Business Ave, Corporate City, US".to_string(),
                billing_address: "456 Business Ave, Corporate City, US".to_string(),
                payment_method: "Business Account".to_string(),
                subtotal: 2999.94,
                tax_amount: 239.99,
                shipping_cost: 0.00,
                total_amount: 3239.93,
                discount_applied: Some("BULK15".to_string()),
                tracking_number: None,
                shipped_date: None,
                delivered_date: None,
            },
        ];

        for order in orders {
            self.order_tree.insert(order.key(), order)?;
        }

        // Create order items
        let order_items = vec![
            OrderItem {
                id: 1,
                order_id: 1,
                product_id: 1,
                quantity: 1,
                unit_price: 1499.99,
                total_price: 1499.99,
                discount_percentage: 10.0,
            },
            OrderItem {
                id: 2,
                order_id: 1,
                product_id: 3,
                quantity: 9,
                unit_price: 24.99,
                total_price: 224.91,
                discount_percentage: 0.0,
            },
            OrderItem {
                id: 3,
                order_id: 2,
                product_id: 1,
                quantity: 2,
                unit_price: 1499.99,
                total_price: 2999.98,
                discount_percentage: 15.0,
            },
        ];

        for item in order_items {
            self.order_item_tree.insert(item.key(), item)?;
        }

        // Create reviews
        let reviews = vec![
            Review {
                id: 1,
                product_id: 1,
                customer_id: 1,
                rating: 5,
                title: "Amazing laptop!".to_string(),
                content: "Perfect for gaming and work. Highly recommended!".to_string(),
                is_verified_purchase: true,
                helpful_votes: 15,
                created_at: 1641427200,
            },
            Review {
                id: 2,
                product_id: 2,
                customer_id: 3,
                rating: 4,
                title: "Great sound quality".to_string(),
                content: "Very good headphones, excellent noise cancellation.".to_string(),
                is_verified_purchase: true,
                helpful_votes: 8,
                created_at: 1641513600,
            },
        ];

        for review in reviews {
            self.review_tree.insert(review.key(), review)?;
        }

        println!("Sample data created successfully!");
        Ok(())
    }
}

// === Data Transfer Objects ===

#[derive(Debug)]
pub struct OrderDetails {
    pub order: Order,
    pub customer: Customer,
    pub items: Vec<OrderItemWithProduct>,
}

#[derive(Debug)]
pub struct OrderItemWithProduct {
    pub item: OrderItem,
    pub product: Product,
}

#[derive(Debug)]
pub struct CustomerLTV {
    pub customer: Customer,
    pub total_spent: f64,
    pub order_count: u32,
    pub avg_order_value: f64,
    pub order_frequency: f64,
    pub predicted_ltv: f64,
    pub days_active: u32,
}

#[derive(Default)]
struct CustomerLTVData {
    pub total_spent: f64,
    pub order_count: u32,
    pub first_order_date: u64,
    pub last_order_date: u64,
}

#[derive(Debug)]
pub struct ProductSearchCriteria {
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub min_rating: Option<f64>,
    pub in_stock_only: bool,
    pub min_review_count: Option<u32>,
    pub brand: Option<String>,
}

#[derive(Debug)]
pub struct RevenueAnalytics {
    pub total_revenue: f64,
    pub total_orders: usize,
    pub avg_order_value: f64,
    pub status_breakdown: HashMap<OrderStatus, usize>,
    pub period_start: u64,
    pub period_end: u64,
}

#[derive(Debug, Default)]
pub struct SalesReport {
    pub total_customers: usize,
    pub total_products: usize,
    pub total_orders: usize,
    pub revenue_by_tier: HashMap<LoyaltyTier, f64>,
    pub top_products: Vec<TopProduct>,
    pub order_status_distribution: HashMap<OrderStatus, usize>,
}

#[derive(Debug)]
pub struct TopProduct {
    pub id: u64,
    pub name: String,
    pub quantity_sold: u32,
    pub revenue: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🚀 Netabase Advanced Queries Example\n");

    // Create the analytics service
    let analytics = EcommerceAnalytics::new("advanced_queries_db")?;

    // Create sample data
    analytics.create_sample_data()?;

    println!("\n=== Advanced Relational Queries ===");

    // Demonstrate complex order details query
    println!("\n1. Complete Order Details Query:");
    match analytics.get_order_details(1) {
        Ok(details) => {
            println!(
                "Order #{}: ${:.2} by {} {}",
                details.order.id,
                details.order.total_amount,
                details.customer.first_name,
                details.customer.last_name
            );
            println!("   Items:");
            for item_detail in &details.items {
                println!(
                    "     - {} x {} @ ${:.2}",
                    item_detail.item.quantity,
                    item_detail.product.name,
                    item_detail.item.unit_price
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Market basket analysis
    println!("\n2. Market Basket Analysis:");
    let product_combo = vec![1, 3]; // Gaming laptop + T-shirt
    match analytics.find_customers_who_bought_together(&product_combo) {
        Ok(customers) => {
            println!(
                "Customers who bought products {:?} together:",
                product_combo
            );
            for customer in customers {
                println!(
                    "   - {} {} ({})",
                    customer.first_name, customer.last_name, customer.email
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Customer Lifetime Value analysis
    println!("\n3. Customer Lifetime Value Analysis:");
    match analytics.analyze_customer_lifetime_value() {
        Ok(mut ltv_analysis) => {
            ltv_analysis.truncate(5); // Show top 5
            for ltv in ltv_analysis {
                println!(
                    "   {} {}: Spent ${:.2}, Predicted LTV ${:.2} ({} orders, avg ${:.2})",
                    ltv.customer.first_name,
                    ltv.customer.last_name,
                    ltv.total_spent,
                    ltv.predicted_ltv,
                    ltv.order_count,
                    ltv.avg_order_value
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Advanced Filtering Queries ===");

    // Complex product search
    println!("\n4. Advanced Product Search:");
    let criteria = ProductSearchCriteria {
        min_price: Some(100.0),
        max_price: Some(2000.0),
        min_rating: Some(4.0),
        in_stock_only: true,
        min_review_count: Some(50),
        brand: None,
    };

    match analytics.find_products_advanced(criteria) {
        Ok(products) => {
            println!("Products matching complex criteria:");
            for product in products {
                println!(
                    "   - {} by {} (${:.2}, ⭐{:.1}, {} reviews)",
                    product.name,
                    product.brand,
                    product.price,
                    product.rating,
                    product.review_count
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // VIP customer identification
    println!("\n5. VIP Customer Identification:");
    match analytics.find_vip_customers() {
        Ok(vip_customers) => {
            println!("VIP Customers (high-value, frequent buyers):");
            for customer in vip_customers {
                println!(
                    "   - {} {} ({:?} tier): ${:.2} spent, {} orders",
                    customer.first_name,
                    customer.last_name,
                    customer.loyalty_tier,
                    customer.total_spent,
                    customer.order_count
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Time-based Range Queries ===");

    // Revenue analytics for date range
    println!("\n6. Revenue Analytics (Last 30 days):");
    let now = chrono::Utc::now().timestamp() as u64;
    let thirty_days_ago = now - (30 * 24 * 3600);

    match analytics.get_revenue_analytics(thirty_days_ago, now) {
        Ok(revenue) => {
            println!("Total Revenue: ${:.2}", revenue.total_revenue);
            println!("Total Orders: {}", revenue.total_orders);
            println!("Average Order Value: ${:.2}", revenue.avg_order_value);
            println!("Order Status Breakdown:");
            for (status, count) in revenue.status_breakdown {
                println!("   {:?}: {}", status, count);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Batch Operations ===");

    // Batch price update
    println!("\n7. Batch Price Update:");
    match analytics.batch_update_prices(1, 5.0) {
        // 5% increase for Electronics category
        Ok(count) => {
            println!(
                "Updated prices for {} products in Electronics category (+5%)",
                count
            );
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n=== Analytics and Reporting ===");

    // Comprehensive sales report
    println!("\n8. Sales Report:");
    match analytics.generate_sales_report() {
        Ok(report) => {
            println!("📊 Sales Dashboard:");
            println!("   Total Customers: {}", report.total_customers);
            println!("   Total Products: {}", report.total_products);
            println!("   Total Orders: {}", report.total_orders);

            println!("\n   Revenue by Loyalty Tier:");
            for (tier, revenue) in report.revenue_by_tier {
                println!("     {:?}: ${:.2}", tier, revenue);
            }

            println!("\n   Top Products:");
            for (i, product) in report.top_products.iter().enumerate() {
                println!(
                    "     {}. {} - {} sold, ${:.2} revenue",
                    i + 1,
                    product.name,
                    product.quantity_sold,
                    product.revenue
                );
            }

            println!("\n   Order Status Distribution:");
            for (status, count) in report.order_status_distribution {
                println!("     {:?}: {}", status, count);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n🎉 Advanced Queries Example Complete!");
    println!("\nAdvanced capabilities demonstrated:");
    println!("  ✅ Complex relational queries with multiple joins");
    println!("  ✅ Market basket analysis and customer behavior");
    println!("  ✅ Customer lifetime value calculations");
    println!("  ✅ Multi-criteria product filtering");
    println!("  ✅ Time-based range queries and analytics");
    println!("  ✅ Batch operations for performance");
    println!("  ✅ Comprehensive reporting and aggregations");
    println!("  ✅ Real-world e-commerce scenario modeling");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_analytics() -> EcommerceAnalytics {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_ecommerce");
        EcommerceAnalytics::new(&db_path.to_string_lossy()).unwrap()
    }

    #[test]
    fn test_order_details_query() {
        let analytics = create_test_analytics();
        analytics.create_sample_data().unwrap();

        let details = analytics.get_order_details(1).unwrap();
        assert_eq!(details.order.id, 1);
        assert_eq!(details.customer.id, 1);
        assert!(!details.items.is_empty());
    }

    #[test]
    fn test_product_search() {
        let analytics = create_test_analytics();
        analytics.create_sample_data().unwrap();

        let criteria = ProductSearchCriteria {
            min_price: Some(100.0),
            max_price: None,
            min_rating: Some(4.0),
            in_stock_only: true,
            min_review_count: None,
            brand: None,
        };

        let products = analytics.find_products_advanced(criteria).unwrap();
        assert!(!products.is_empty());
        for product in products {
            assert!(product.price >= 100.0);
            assert!(product.rating >= 4.0);
            assert!(product.in_stock);
        }
    }

    #[test]
    fn test_ltv_analysis() {
        let analytics = create_test_analytics();
        analytics.create_sample_data().unwrap();

        let ltv_results = analytics.analyze_customer_lifetime_value().unwrap();
        assert!(!ltv_results.is_empty());

        for ltv in ltv_results {
            assert!(ltv.total_spent > 0.0);
            assert!(ltv.order_count > 0);
            assert!(ltv.avg_order_value > 0.0);
        }
    }

    #[test]
    fn test_sales_report() {
        let analytics = create_test_analytics();
        analytics.create_sample_data().unwrap();

        let report = analytics.generate_sales_report().unwrap();
        assert!(report.total_customers > 0);
        assert!(report.total_products > 0);
        assert!(report.total_orders > 0);
        assert!(!report.revenue_by_tier.is_empty());
    }
}
