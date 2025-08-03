# Library Management System

A comprehensive library management system demonstrating the full capabilities of graph-sql with various SQLite data types and complex relationships.

## Overview

This example showcases a complete library management system with:
- **Authors** and **Publishers** with biographical information
- **Books** with detailed metadata (ISBN, ratings, availability)
- **Members** with different membership types and borrowing limits
- **Loans** with due dates, renewals, and fine tracking
- **Reservations** for when books are unavailable
- **Reviews** by members with ratings and helpful votes
- **Genres** with hierarchical parent-child relationships

## Database Schema Features

### Data Type Demonstrations
- **TEXT**: Names, descriptions, addresses, ISBNs
- **INTEGER**: IDs, years, page counts, quantities
- **REAL**: Prices, weights, fine amounts
- **NUMERIC**: Precise ratings (e.g., NUMERIC(3,2) for author ratings)
- **BLOB**: Images, files, binary data (mapped to String in GraphQL)
- **BOOLEAN**: Active status, flags, toggles

### Relationship Patterns
- **One-to-Many**: Author → Books, Publisher → Books, Member → Loans
- **Many-to-One**: Books → Genre, Loans → Book/Member
- **Self-Referencing**: Genre → Parent Genre (hierarchical categories)
- **Unique Constraints**: ISBNs, emails, membership numbers

### Business Logic Constraints
- Check constraints for valid ratings, positive quantities
- Foreign key constraints with CASCADE deletes where appropriate
- Default values for timestamps, status flags
- Complex constraints (available_copies <= total_copies)

## Running the Example

1. **Start the server**:
   ```bash
   cd examples/library
   cargo run
   ```

2. **Apply migrations**:
   ```bash
   sqlx migrate run --database-url sqlite:library.db
   ```

3. **Access GraphiQL**: Open [http://localhost:8083/graphiql](http://localhost:8083/graphiql)

## Example Queries

### Basic Book Queries

```graphql
# Get all books with author and publisher information
query GetAllBooks {
  books {
    id
    title
    isbn
    rating
    availableCopies
    author {
      name
      nationality
    }
    publisher {
      name
      website
    }
    genre {
      name
      description
    }
  }
}

# Search books by author nationality
query GetBritishAuthors {
  authors(where: "nationality = 'British'") {
    name
    birthYear
    books {
      title
      publicationYear
      rating
    }
  }
}
```

### Complex Relationship Queries

```graphql
# Get members with their current loans and overdue status
query GetMemberLoans {
  members {
    firstName
    lastName
    membershipType
    fineAmount
    loans(where: "is_returned = 0") {
      id
      loanDate
      dueDate
      book {
        title
        author {
          name
        }
      }
    }
  }
}

# Get books with reviews and average ratings
query GetBooksWithReviews {
  books {
    title
    rating
    reviews {
      rating
      title
      content
      helpfulVotes
      member {
        firstName
        lastName
      }
    }
  }
}
```

### Hierarchical Genre Queries

```graphql
# Get genre hierarchy
query GetGenreHierarchy {
  genres(where: "parent_genre_id IS NULL") {
    name
    description
    # Note: Nested genres would require recursive resolvers
    books {
      title
      author {
        name
      }
    }
  }
}
```

### Availability and Reservation Queries

```graphql
# Check book availability and reservations
query GetBookAvailability {
  books {
    title
    totalCopies
    availableCopies
    loans(where: "is_returned = 0") {
      member {
        firstName
        lastName
      }
      dueDate
    }
    reservations(where: "is_fulfilled = 0 AND is_cancelled = 0") {
      member {
        firstName
        lastName
      }
      reservationDate
      expiryDate
    }
  }
}
```

## Example Mutations

### Adding New Books

```graphql
mutation AddNewBook {
  createBook(input: {
    isbn: "978-1234567890"
    title: "New Science Fiction Novel"
    authorId: 4
    publisherId: 1
    genreId: 3
    publicationYear: 2024
    pageCount: 350
    price: 24.99
    totalCopies: 3
    availableCopies: 3
    description: "An exciting new sci-fi adventure"
  }) {
    id
    title
    isbn
  }
}
```

### Member Registration

```graphql
mutation RegisterMember {
  createMember(input: {
    membershipNumber: "LIB006"
    firstName: "John"
    lastName: "Doe"
    email: "john.doe@email.com"
    phone: "+1-555-0106"
    membershipType: "standard"
    expiryDate: "2025-12-31"
  }) {
    id
    membershipNumber
    firstName
    lastName
  }
}
```

### Book Borrowing Workflow

```graphql
# 1. Create a loan
mutation BorrowBook {
  createLoan(input: {
    bookId: 1
    memberId: 2
    dueDate: "2024-02-15"
  }) {
    id
    loanDate
    dueDate
    book {
      title
    }
    member {
      firstName
      lastName
    }
  }
}

# 2. Return a book
mutation ReturnBook {
  updateLoan(id: 1, input: {
    returnDate: "2024-02-10"
    isReturned: true
  }) {
    id
    returnDate
    fineAmount
  }
}
```

### Review System

```graphql
mutation AddBookReview {
  createReview(input: {
    bookId: 1
    memberId: 1
    rating: 5
    title: "Absolutely fantastic!"
    content: "This book exceeded all my expectations. Highly recommended!"
  }) {
    id
    rating
    title
    reviewDate
    book {
      title
    }
    member {
      firstName
      lastName
    }
  }
}
```

## Data Type Mappings Demonstrated

| SQLite Type | GraphQL Type | Example Field | Notes |
|-------------|--------------|---------------|-------|
| INTEGER PRIMARY KEY | ID! | `id` | Auto-increment |
| TEXT | String | `name`, `email` | Variable length |
| INTEGER | Int | `publicationYear`, `pageCount` | 32-bit signed |
| REAL | Float | `price`, `weight` | Double precision |
| NUMERIC(3,2) | String | `rating` | Precise decimals |
| BLOB | String | `coverImage`, `digitalFile` | Base64 encoded |
| BOOLEAN | Boolean | `isActive`, `isReturned` | True/false |
| TEXT (datetime) | String | `createdAt`, `dueDate` | ISO format |

## Foreign Key Relationships

The system automatically detects and maps foreign key relationships:

- `author_id` → `author` field (many-to-one)
- `publisher_id` → `publisher` field (many-to-one)  
- `genre_id` → `genre` field (many-to-one)
- `book_id` → `book` field (many-to-one)
- `member_id` → `member` field (many-to-one)
- `parent_genre_id` → `parentGenre` field (self-referencing)

Reverse relationships are also created:
- `author.books` (one-to-many)
- `member.loans` (one-to-many)
- `book.reviews` (one-to-many)

## Business Logic Examples

### Membership Types and Limits
- **Standard**: 5 books max
- **Premium**: 10 books max  
- **Student**: 8 books max
- **Senior**: 7 books max

### Fine Calculation
- Overdue books accrue fines automatically
- Fine amounts are tracked per loan
- Members with outstanding fines may have borrowing restrictions

### Reservation System
- Members can reserve books that are currently checked out
- Reservations have expiry dates
- Automatic notifications when reserved books become available

This example demonstrates how graph-sql can handle complex business domains with rich data types, intricate relationships, and real-world constraints while providing a clean GraphQL API for all operations.
