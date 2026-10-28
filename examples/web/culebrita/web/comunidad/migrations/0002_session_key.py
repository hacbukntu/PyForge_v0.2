"""Añade aislamiento por sesión sin borrar las entradas existentes."""
from django.db import migrations, models
class Migration(migrations.Migration):
    dependencies = [('comunidad', '0001_initial')]
    operations = [migrations.AddField(model_name='savedentry', name='session_key', field=models.CharField(max_length=64, blank=True, default='', db_index=True))]
