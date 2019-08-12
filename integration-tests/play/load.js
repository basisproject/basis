const config = require('../helpers/config');
const protobuf = require('../helpers/protobuf');

protobuf.load()
	.then(() => {
		const Company = protobuf.protos.company.lookupType('basis.company.Company');
	})
	.catch((err) => {
		console.error('err: ', err.stack);
		throw err;
	});

