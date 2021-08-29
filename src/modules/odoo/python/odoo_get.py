import json
import sys
import jsonpickle
from datetime import timedelta
from datetime import datetime

url = str(sys.argv[1])
db = str(sys.argv[2])
username = str(sys.argv[3])
password = str(sys.argv[4])

import xmlrpc.client

class Project(object):
        id :str = ""
        name :str = ""
        description :str = ""
        needs = []

class ProjectNeed(object):
        label :str = ""
        nbr :int = ""
        begin :datetime
        end :datetime
        skills = []

class ProjectNeedSkill(object):
        skill :str = ""
        level :str = ""
        min_years :int
        mandatory :bool

t = datetime.now() - timedelta(days=10)
limit = t.strftime('%Y-%m-%d %H:%M:%S')

common = xmlrpc.client.ServerProxy('{}/xmlrpc/2/common'.format(url))

uid = common.authenticate(db, username, password, {})

models = xmlrpc.client.ServerProxy('{}/xmlrpc/2/object'.format(url))
c = models.execute_kw(db, uid, password,
    'crm.lead', 'search_read',
    [[['write_date', '>', limit]]],
    {'fields': ['id', 
        'name',
        'x_studio_hula_project_name', 
        'x_studio_description', 
        'x_studio_begin', 
        'x_studio_end', 
        'x_studio_nbr_of_positions', 
        'x_studio_skill', 
        'x_studio_level', 
        'x_studio_minimum_years', 
        'x_studio_mandatory', 
        'x_studio_skill_2', 
        'x_studio_level_2', 
        'x_studio_minimum_years_2', 
        'x_studio_mandatory_2'],
        'limit': 5000})

res = []

for cc in c:
    if not cc['x_studio_hula_project_name']:
            continue

    if not cc['x_studio_skill']:
            continue

    skills = []
    if cc['x_studio_skill']:
        skill = ProjectNeedSkill()
        skill.skill = cc['x_studio_skill'][1]
        skill.level = cc['x_studio_level'][1]
        skill.min_years = cc['x_studio_minimum_years'] 
        skill.mandatory = cc['x_studio_mandatory']
        skills.append(skill)

    if cc['x_studio_skill_2']:
        skill2 = ProjectNeedSkill()
        skill2.skill = cc['x_studio_skill_2'][1]
        skill2.level = cc['x_studio_level_2'][1]
        skill2.min_years = cc['x_studio_minimum_years_2'] 
        skill2.mandatory = cc['x_studio_mandatory_2']
        skills.append(skill2)

    needs = []
    need = ProjectNeed()
    need.label = cc['x_studio_hula_project_name']
    need.nbr = cc['x_studio_nbr_of_positions']
    need.begin = cc['x_studio_begin']
    need.end = cc['x_studio_end']
    need.skills = skills

    proj = Project()
    proj.id = cc['id']
    proj.name = cc['x_studio_hula_project_name']
    proj.description = cc['x_studio_description']
    proj.needs = [need]

    res.append(proj)

print(jsonpickle.encode(res, unpicklable=False))

#for r in res:
#    print(jsonpickle.encode(r, unpicklable=False))

