# Basis

This is a blockchain implementation of the system described in the [basis paper](https://gitlab.com/basis-/paper). The system's processes and decisions will not be described here as the paper will do a better job. Here you will find technical information on using/running the project.

## What is Basis?

Basis is a blockchain implementation of a cost-tracking distributed product network. Think of it like a moneyless Amazon that tracks costs of production in labor hours and resources.

For instance, if your company makes chairs in a socialist economy, you would use Basis to track the labor time of the workers of your company, to order inputs to production (such as lumber), and to allow other companies or end-consumers to order your products from you.

The purpose of Basis is to calculate the exact costs of production (both costs of labor and (semi-)raw materials like lumber or iron) of each participating company over time. The removal of the market concept of "pricing" from the productive process allows calculation of a disaggregate cost of each product, which can be used by producers, economic planners, or consumers.

## Why build it?

Whether using moneyless markets, decentralized planning, or fully centralized planning, all socialist economies have one thing in common: they need to track labor and to some extend resource usage. Basis is not only the economic framework for tracking labor costs, but also the network that companies in a socialist economy use to transact.

It's a completely transparent decentralized system that accounts for all costs of production.

## How does it work?

Basis has three main parts:

1. Labor tracking. Workers of a company clock in/out, and Basis keeps track of their hours.
2. Products and services. Companies provide products and services to other companies, who order them through Basis. Think of it like a socialist/moneyless Amazon.
3. Orders. Companies order products and services from each other, and each of these have a cost (in labor and resources) associated with it. The costs of a company's products are essentially the `(costs of labor + costs of outgoing orders) / products produced`.

Unifying all three of these concepts on the same platform allows accurate cost tracking.

# Documentation

Basis is made of of the following concepts:

- [Users](#users)
- [Companies](#companies)
- [Members](#members)
- [Products](#products)
- [Services](#services)
- [Labor](#labor)
- [Orders](#orders)
- [Amortization](#amortization)
- [Resource tags](#resource-tags)
- [Cost tags](#cost-tags)

## Users

A User in Basis is essentially an ID, a public key, and a set of roles. Users are the most fundamental building block in the system, as all other models rely on them in some way.

## Companies

A company is a basic object consisting of an ID, email, and name. Although company objects don't store much data, they are central to the rest of the system. Companies are the owners of [company members](#members) (workers), [products](#products), [labor](#labor), and [cost tags](#cost-tags).

Companies are the object that most of the economic and tracking data attaches to, and act as a sort of container for costs. Companies also have a specific set of per-member permissions that control who can do what Basis-realted tasks (such as handling orders, managing labor tracking, tagging costs, etc). A company object in Basis would be the same as a company in the market economy (like "Amazon" or "Safeway").

## Members

A member is a worker of a company, and is a record in the system that links a [user](#users) to a [company](#company), as well as contains a set of roles that that user can assume in the context of the company. A member would be like an "employee" in the market system (except they would be a worker-owner of the company, as opposed to exchanging their labor for a wage).

## Products

A product in Basis is what a company builds and can be ordered by other companies. A product could be a chair, a watt-hour of electricity, a sandwich, etc. Products are given specific dimensions (if applicable), mass (if applicable), and a unit of measurement (millimeter, milliliter, watt-hour, etc).

Products are building blocks of the cost tracking system. They have costs attached to them directly, which are updated each time some new company activity occurs (such as an order or new labor record being created).

## Services

TODO (see [tracker/#20](https://gitlab.com/basis-/tracker/issues/20)).

## Labor

Labor records are attached to a [member](#members) record and allows a company to track the costs of labor. Each time a member clocks in, a new labor record is created, and each time they clock out, the existing labor record is updated to reflect the exact time of the working period.

Labor records are used in cost calculations to determine costs of [products](#products) and [services](#services).

## Orders

Orders are the fabric that tie [companies](#companies), [products](#products), and [services](#services) together in the Basis system.

An order is an item that records an originating company ("purchaser" in market terms), receiving company ("seller"), and a set of products (provided by the "seller") along with their costs and quantities.

Orders (along with labor records) are the main input to the cost calculations.

It's important to note that in the market system, orders generally require some form of exchange to occur (for instance, if you buy 500 widgets, you have to pay for them). Basis has no built-in mechanism for payment or exchange. More on this in the [costs section of the documentation](#costs).

## Amortization

TODO (see [tracker/#21](https://gitlab.com/basis-/tracker/issues/21)).

## Resource tags

A resource tag is a special marker that attaches to a [product](#products) and tells Basis it should be tracked individually. For instance, a resource tag might be used on a company that mines silicon, and as that silicon moves through the economy, Basis will track it in its own cost "bucket" (as opposed to just tracking the labor used to extract the silicon).

In other words, resource tags transform a product, which is generally tracked just in costs of labor, into a resource which is worthy of tracking on its own.

## Cost tags

Cost tags are how companies internally account for costs. The way Basis works, *all costs must be accounted for*, however there has to be some way to tell the system which costs go to which products and services. This is where cost tags come in.

Cost tags can be attached to labor records, orders, products, and services, and are used to create different cost buckets in the company that are then divvied up to the various products and services the company provides. More details on the algorithm can be found in the [costs sections of the documentation](#costs).

# Costs

# Running

## Building

Requires Rust v1.34+. To build the project, run:

```
make
```

To configure, run:

```
make reconfig
```

Now run it:

```
make run
```

Congrats, you are running a Basis node.

## Testing

Basis comes with a series of built-in unit tests that can be run via:

```
make test
```

It also comes with a set of [integration tests as an included project](./integration-tests).


## License

Basis uses AGPLv3.0, which means not only is it a copyleft license, but anyone running the server must also provide/publish the source to users of their server on request.

Basis, at its core, is about complete transparency and openness. The AGPL license reflects and to some extent enforces this.



