const config = require('./config');
const rp = require('request-promise');

exports.list = async function({after, per_page}) {
	return rp({
		url: `${config.endpoint}/services/factor/v1/users`,
		json: true,
		qs: {
			after: after,
			per_page: per_page,
		},
	});
};

exports.get = async function({id, pubkey, email}, options) {
	options || (options = {});
	const {extended} = options;
	let res = await rp({
		url: `${config.endpoint}/services/factor/v1/users/info`,
		json: true,
		qs: {
			id: id,
			pubkey: pubkey,
			email: email,
		},
	});
	if(!extended) res = res.user;
	return res;
};

