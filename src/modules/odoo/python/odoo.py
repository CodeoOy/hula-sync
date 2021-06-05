import json
import sys

url = str(sys.argv[1])
db = str(sys.argv[2])
username = str(sys.argv[3])
password = str(sys.argv[4])

import xmlrpc.client

common = xmlrpc.client.ServerProxy('{}/xmlrpc/2/common'.format(url))

uid = common.authenticate(db, username, password, {})

models = xmlrpc.client.ServerProxy('{}/xmlrpc/2/object'.format(url))
c = models.execute_kw(db, uid, password,
    'crm.lead', 'search_read',
    [[]],
    {'fields': ['name'], 'limit': 5})

print(json.dumps(c))

