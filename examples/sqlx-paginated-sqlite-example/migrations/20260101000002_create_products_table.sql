-- Create products table
CREATE TABLE IF NOT EXISTS products (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    price REAL NOT NULL,
    stock INTEGER NOT NULL,
    category TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL
);

-- Create index on category for filtering
CREATE INDEX IF NOT EXISTS idx_products_category ON products(category);

-- Create index on status for filtering
CREATE INDEX IF NOT EXISTS idx_products_status ON products(status);

-- Create index on price for filtering and sorting
CREATE INDEX IF NOT EXISTS idx_products_price ON products(price);

-- Create index on stock for filtering
CREATE INDEX IF NOT EXISTS idx_products_stock ON products(stock);

-- Create composite index for common queries
CREATE INDEX IF NOT EXISTS idx_products_category_status ON products(category, status);


