-- Seed test data for users
INSERT INTO users (id, first_name, last_name, email, confirmed, created_at, updated_at)
SELECT * FROM (
    SELECT '550e8400-e29b-41d4-a716-446655440001' as id, 'John' as first_name, 'Smith' as last_name, 'john.smith@example.com' as email, 1 as confirmed, datetime('now') as created_at, datetime('now') as updated_at
    UNION ALL SELECT '550e8400-e29b-41d4-a716-446655440002', 'Jane', 'Doe', 'jane.doe@example.com', 1, datetime('now'), datetime('now')
    UNION ALL SELECT '550e8400-e29b-41d4-a716-446655440003', 'Johnny', 'Appleseed', 'johnny.appleseed@example.com', 0, datetime('now'), datetime('now')
    UNION ALL SELECT '550e8400-e29b-41d4-a716-446655440004', 'Alice', 'Johnson', 'alice.johnson@example.com', 1, datetime('now'), datetime('now')
    UNION ALL SELECT '550e8400-e29b-41d4-a716-446655440005', 'Bob', 'Williams', 'bob.williams@example.com', 0, datetime('now'), datetime('now')
    UNION ALL SELECT '550e8400-e29b-41d4-a716-446655440006', 'Charlie', 'Brown', 'charlie.brown@example.com', 1, datetime('now'), datetime('now')
    UNION ALL SELECT '550e8400-e29b-41d4-a716-446655440007', 'Diana', 'Prince', 'diana.prince@example.com', 1, datetime('now'), datetime('now')
    UNION ALL SELECT '550e8400-e29b-41d4-a716-446655440008', 'Eve', 'Anderson', 'eve.anderson@example.com', 0, datetime('now'), datetime('now')
    UNION ALL SELECT '550e8400-e29b-41d4-a716-446655440009', 'Frank', 'Miller', 'frank.miller@example.com', 1, datetime('now'), datetime('now')
    UNION ALL SELECT '550e8400-e29b-41d4-a716-446655440010', 'Grace', 'Wilson', 'grace.wilson@example.com', 1, datetime('now'), datetime('now')
) WHERE NOT EXISTS (SELECT 1 FROM users);

-- Seed test data for products
INSERT INTO products (id, name, description, price, stock, category, status, created_at)
SELECT * FROM (
    SELECT '650e8400-e29b-41d4-a716-446655440001' as id, 'Laptop Pro' as name, 'High-performance laptop' as description, 1299.99 as price, 15 as stock, 'computers' as category, 'active' as status, datetime('now') as created_at
    UNION ALL SELECT '650e8400-e29b-41d4-a716-446655440002', 'Wireless Mouse', 'Ergonomic wireless mouse', 29.99, 50, 'electronics', 'active', datetime('now')
    UNION ALL SELECT '650e8400-e29b-41d4-a716-446655440003', 'Mechanical Keyboard', 'RGB mechanical keyboard', 149.99, 30, 'electronics', 'active', datetime('now')
    UNION ALL SELECT '650e8400-e29b-41d4-a716-446655440004', 'USB-C Hub', '7-in-1 USB-C hub', 49.99, 100, 'accessories', 'active', datetime('now')
    UNION ALL SELECT '650e8400-e29b-41d4-a716-446655440005', 'Monitor 27"', '4K UHD monitor', 399.99, 25, 'computers', 'active', datetime('now')
    UNION ALL SELECT '650e8400-e29b-41d4-a716-446655440006', 'Webcam HD', '1080p webcam', 79.99, 40, 'electronics', 'active', datetime('now')
    UNION ALL SELECT '650e8400-e29b-41d4-a716-446655440007', 'Laptop Stand', 'Adjustable aluminum stand', 39.99, 60, 'accessories', 'active', datetime('now')
    UNION ALL SELECT '650e8400-e29b-41d4-a716-446655440008', 'External SSD 1TB', 'Portable SSD storage', 119.99, 35, 'computers', 'active', datetime('now')
    UNION ALL SELECT '650e8400-e29b-41d4-a716-446655440009', 'Desk Lamp', 'LED desk lamp', 34.99, 45, 'accessories', 'active', datetime('now')
    UNION ALL SELECT '650e8400-e29b-41d4-a716-446655440010', 'Headphones', 'Noise-cancelling headphones', 299.99, 20, 'electronics', 'active', datetime('now')
    UNION ALL SELECT '650e8400-e29b-41d4-a716-446655440011', 'Phone Holder', 'Adjustable phone holder', 15.99, 80, 'accessories', 'discontinued', datetime('now')
    UNION ALL SELECT '650e8400-e29b-41d4-a716-446655440012', 'Cable Organizer', 'Cable management system', 19.99, 0, 'accessories', 'out_of_stock', datetime('now')
) WHERE NOT EXISTS (SELECT 1 FROM products);

