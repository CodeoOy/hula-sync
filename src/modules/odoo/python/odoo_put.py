import json
import sys

url = str(sys.argv[1])
db = str(sys.argv[2])
username = str(sys.argv[3])
password = str(sys.argv[4])
matches = json.loads(str(sys.argv[5]))

import xmlrpc.client

common = xmlrpc.client.ServerProxy('{}/xmlrpc/2/common'.format(url))

uid = common.authenticate(db, username, password, {})

models = xmlrpc.client.ServerProxy('{}/xmlrpc/2/object'.format(url))

for match in matches:
        nbr_of_matches = match["matches"]
        link = '<a href="' + match["link"] + '" target="_blank">Link to project comes here</a>'

        existing = models.execute_kw(db, uid, password,
                'crm.lead', 'search_read',
                [[['id', '=', match["id"]]]],
                {'fields': ['x_studio_nbr_of_matches', 
                        'x_studio_link']})

        if existing:
                if existing['x_studio_nbr_of_matches'] = nbr_of_matches and existing['x_studio_link']:
                        continue

        models.execute_kw(db, uid, password, 'crm.lead', 'write', [[match["id"]], {
                'x_studio_nbr_of_matches': nbr_of_matches,
                'x_studio_link': link
        }])
