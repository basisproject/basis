/**
 * The purpose of this module is to grab the orders, labor, and products for a
 * company from the (running) Basis API and convert them into code that can be
 * copy/pasted into the product costs tests.
 *
 * The reason it's done this way (as opposed to having the test call the api
 * directly) is because in the early stages of debugging the costing algorithm,
 * it can really help to cherry(, babyyy) pick the orders we want to go into the
 * costs by commenting certain ones out to see how they affect the end result.
 *
 * So, yes, it would be more "correct" to load the tests in rust and deserialize
 * them direct from the API but this method gives us just a bit more precision.
 */

const rp = require('request-promise');

const api_endpoint = process.env['ENDPOINT'] || 'http://127.0.0.1:13007/api';

const print_date = (dateobj) => `time::from_timestamp(${dateobj.seconds})`;
const print_enum = (enumval) => enumval[0].toUpperCase() + enumval.substr(1).toLowerCase();

function usage() {
	console.log('Usage: node generate-entity-test-code.js <company id>')
	process.exit(1);
}

function print_orders(orders, fn) {
	console.log(`    fn test_orders_${fn}() -> Vec<Order> {`);
	console.log('        let fakehash = make_hash();');
	console.log('        vec![');
	orders.forEach((o) => console.log('            '+`
		Order::new(
			"${o.id}",
			"${o.company_id_from}",
			"${o.company_id_to}",
			&CostCategory::${print_enum(o.cost_category)},
			&vec![${o.products.map((p) => `{
				let mut costs = Costs::new();
				${Object.keys(p.costs.labor).map((c) => `costs.track_labor("${c}", ${p.costs.labor[c]} as f64);`).join(' ')}
				${Object.keys(p.costs.products).map((c) => `costs.track("${c}", ${p.costs.products[c]} as f64);`).join(' ')}
				ProductEntry::new("${p.product_id}", ${p.quantity} as f64, &costs, ${p.resource})
			}`).join(', ')}],
			&ProcessStatus::${print_enum(o.process_status)},
			&${print_date(o.created)},
			&${print_date(o.updated)},
			1,
			&fakehash
		),
	`.replace(/(^\s+|\n|\s$)/g, '').replace(/\s+/g, ' ')));
	console.log('        ]');
	console.log(`    }`);
}


function print_labor(labor) {
	console.log(`    fn test_labor() -> Vec<Labor> {`);
	console.log('        let fakehash = make_hash();');
	console.log('        vec![');
	labor.forEach((l) => console.log('            '+`
		Labor::new(
			"${l.id}",
			"${l.company_id}",
			"${l.user_id}",
			"${l.occupation}",
			Some(&${print_date(l.start)}),
			Some(&${print_date(l.end)}),
			&${print_date(l.created)},
			&${print_date(l.updated)},
			1,
			&fakehash
		),
	`.replace(/(^\s+|\n|\s$)/g, '').replace(/\s+/g, ' ')));
	console.log('        ]');
	console.log(`    }`);
}

function print_products(products) {
	const units = {
        MILLIMETER: 'Millimeter',
        MILLILITER: 'Milliliter',
        WATTHOUR: 'WattHour',
        EACH: 'Each',
	};
	const effort = {
        NANOSECONDS: 'Nanoseconds',
        MILLISECONDS: 'Milliseconds',
        SECONDS: 'Seconds',
        MINUTES: 'Minutes',
        HOURS: 'Hours',
        DAYS: 'Days',
        WEEKS: 'Weeks',
        YEARS: 'Years',
	};
	console.log(`    fn test_products() -> HashMap<String, Product> {`);
    console.log('        let fakehash = make_hash();');
    console.log('        let mut products = HashMap::new();');
	products.map((p) => p.product).forEach((p) => console.log('        '+`
		products.insert("${p.id}".to_owned(), Product::new(
			"${p.id}",
			"${p.company_id}",
			"${p.name}",
			&Unit::${units[p.unit]},
			${p.mass_mg} as f64,
			&Dimensions::new(
				${p.dimensions.width} as f64,
				${p.dimensions.height} as f64,
				${p.dimensions.length} as f64
			),
			&vec![${p.inputs.map((i) => `Input::new("${i.product_id}", ${i.quantity} as f64)`).join(', ')}],
			&Effort::new(&EffortTime::${effort[p.effort.time]}, ${p.effort.quantity} as u64),
			${p.active},
			"${p.meta}",
			&${print_date(p.created)},
			&${print_date(p.updated)},
			${p.deleted.seconds > 0 ? `Some(&print_date(p.deleted))` : `None`},
			1,
			&fakehash
		));
	`.replace(/(^\s+|\n|\s$)/g, '').replace(/\s+/g, ' ')));
    console.log('        products');
	console.log(`    }`);
}

async function grab_orders(company_id) {
	return rp({url: `${api_endpoint}/services/basis/v1/orders/company-current`, qs: {company_id: company_id}, json: true});
}

async function grab_labor(company_id) {
	var res = await rp({url: `${api_endpoint}/services/basis/v1/labor/company-current`, qs: {company_id: company_id}, json: true});
	return res;
}

async function grab_products(company_id) {
	var res = await rp({url: `${api_endpoint}/services/basis/v1/products/by-company`, qs: {company_id: company_id}, json: true});
	return res.items;
}

async function main() {
	const company_id = process.argv[2];
	if(!company_id) return usage();

	let orders = await grab_orders(company_id);
	let labor = await grab_labor(company_id);
	let products = await grab_products(company_id);

	print_orders(orders.incoming, 'incoming');
	console.log('');
	print_orders(orders.outgoing, 'outgoing');
	console.log('');
	print_labor(labor);
	console.log('');
	print_products(products);
}

main()
	.catch((err) => console.log('err: ', err.stack))
	.finally(() => setTimeout(process.exit, 300));

