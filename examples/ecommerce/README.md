# E-commerce System Example

This example demonstrates a comprehensive e-commerce system with customers, products, orders, reviews, and shopping carts. It showcases graph-sql's ability to handle complex business relationships and data integrity constraints.

## Features Demonstrated

- **Complex Business Logic**: Product catalog, order management, customer relationships
- **Hierarchical Data**: Category trees with parent-child relationships
- **Multiple Address Types**: Billing and shipping addresses per customer
- **Order Management**: Complete order lifecycle with items, pricing, and status tracking
- **Shopping Cart**: Session-based cart functionality
- **Product Reviews**: Customer feedback system with ratings and verification
- **Data Constraints**: Price validations, stock management, order status workflows
- **Financial Calculations**: Tax, shipping, discounts, and totals

## Schema Overview

```sql
category (hierarchical product categories)
├── product (catalog items with inventory)
│   ├── order_item (products in orders)
│   ├── review (customer reviews)
│   └── cart_item (shopping cart contents)
├── customer (user accounts)
│   ├── address (billing/shipping addresses)
│   ├── order (purchase orders)
│   ├── review (product reviews)
│   └── cart_item (shopping cart)
└── order (purchase transactions)
    └── order_item (line items)
```

## Running the Example

```bash
cd examples/ecommerce
cargo run --bin shop
```

The server will start on `http://localhost:8081` with GraphiQL available for testing.

## Example Queries

### Browse product catalog with categories
```graphql
{
  product {
    list(input: {page: 1, limit: 8}) {
      id
      name
      price
      stock_quantity
      sku
      brand
      category {
        name
        description
        parent_category {
          name
        }
      }
    }
  }
}
```

### View customer orders with details
```graphql
{
  order {
    list(input: {page: 1, limit: 5}) {
      order_number
      status
      total_amount
      payment_status
      customer {
        first_name
        last_name
        email
      }
      billing_address {
        street
        city
        state
      }
      shipping_address {
        street
        city
        state
      }
    }
  }
}
```

### Get order items with product details
```graphql
{
  order_item {
    list(input: {page: 1, limit: 10}) {
      quantity
      unit_price
      total_price
      order {
        order_number
        status
        customer {
          first_name
          last_name
        }
      }
      product {
        name
        sku
        category {
          name
        }
      }
    }
  }
}
```

### Product reviews with customer info
```graphql
{
  review {
    list(input: {page: 1, limit: 10}) {
      rating
      title
      content
      is_verified_purchase
      product {
        name
        price
      }
      customer {
        first_name
        last_name
      }
    }
  }
}
```

### Customer shopping cart contents
```graphql
{
  cart_item {
    list(input: {page: 1, limit: 20}) {
      quantity
      customer {
        first_name
        last_name
        email
      }
      product {
        name
        price
        stock_quantity
        category {
          name
        }
      }
    }
  }
}
```

## Example Mutations

### Create a new customer
```graphql
mutation {
  insert_customer(input: {
    first_name: "Alice"
    last_name: "Cooper"
    email: "alice.cooper@example.com"
    phone: "+1-555-0199"
  }) {
    id
    first_name
    last_name
    email
    loyalty_points
  }
}
```

### Add a product to cart
```graphql
mutation {
  insert_cart_item(input: {
    customer_id: 1
    product_id: 3
    quantity: 1
  }) {
    id
    quantity
    customer {
      first_name
      last_name
    }
    product {
      name
      price
    }
  }
}
```

### Create a new order
```graphql
mutation {
  insert_order(input: {
    customer_id: 1
    billing_address_id: 1
    shipping_address_id: 1
    order_number: "ORD-2025-999"
    subtotal: 999.99
    tax_amount: 80.00
    shipping_amount: 12.99
    total_amount: 1092.98
    status: "confirmed"
  }) {
    id
    order_number
    total_amount
    customer {
      first_name
      last_name
    }
  }
}
```

### Add a product review
```graphql
mutation {
  insert_review(input: {
    product_id: 1
    customer_id: 2
    rating: 5
    title: "Amazing laptop!"
    content: "This MacBook Pro exceeded all my expectations. Highly recommended!"
    is_verified_purchase: true
  }) {
    id
    rating
    title
    product {
      name
    }
    customer {
      first_name
      last_name
    }
  }
}
```

## Business Logic Features

- **Inventory Management**: Stock quantities and low stock thresholds
- **Pricing Structure**: Cost price, selling price, and margin tracking
- **Order Workflow**: Status progression from pending to delivered
- **Address Management**: Multiple addresses per customer with type designation
- **Payment Tracking**: Separate payment status from order status
- **Customer Loyalty**: Points system for repeat customers
- **Product Organization**: Hierarchical categories with parent-child relationships
- **Review System**: Verified purchase reviews with approval workflow
- **Shopping Cart**: Session-based cart with quantity management

This example demonstrates how graph-sql handles real-world e-commerce complexity with automatic relationship mapping, data integrity, and comprehensive CRUD operations across all business entities.
