"""Regresiones del foro: autorización, XSS, CSRF, imágenes, SQL y contraseñas."""
import io
from django.contrib.auth import get_user_model
from django.core.files.uploadedfile import SimpleUploadedFile
from django.test import TestCase, Client
from PIL import Image
from .models import Topic, Note
from .forms import TopicForm


class ForumTests(TestCase):
    """Solo cuentas de prueba y base SQL temporal del framework."""
    @classmethod
    def setUpTestData(cls):
        cls.first = get_user_model().objects.create_user('ana', password='Prueba-larga!1357')
        cls.second = get_user_model().objects.create_user('luis', password='Otra-prueba!2468')

    def test_private_routes_and_csrf(self):
        self.assertEqual(self.client.get('/foro/').status_code,302)
        strict = Client(enforce_csrf_checks=True)
        self.assertEqual(strict.post('/foro/registro/', {'username':'extra'}).status_code,403)
        strict.force_login(self.first)
        self.assertEqual(strict.post('/notas/', {'text':'sin csrf'}).status_code,403)
        self.assertEqual(Note.objects.count(),0)

    def test_registration_hash_and_login_limits(self):
        response = self.client.post('/foro/registro/', {'username':'nueva','password1':'Suficiente!9876nuevo','password2':'Suficiente!9876nuevo'})
        self.assertEqual(response.status_code,302)
        user=get_user_model().objects.get(username='nueva')
        self.assertNotEqual(user.password,'Suficiente!9876nuevo')
        self.assertTrue(user.check_password('Suficiente!9876nuevo'))
        self.client.logout()
        for _ in range(8): self.client.post('/foro/entrar/', {'username':'ana','password':'incorrecta'})
        self.assertEqual(self.client.post('/foro/entrar/', {'username':'ana','password':'incorrecta'}).status_code,429)

    def test_xss_and_owner_sql(self):
        self.client.force_login(self.first)
        self.client.post('/notas/', {'text':"'; DROP TABLE comunidad_note; --"})
        self.assertEqual(Note.objects.filter(owner=self.first).count(),1)
        Note.objects.create(owner=self.second,text='NOTA-AJENA')
        response=self.client.get('/notas/')
        self.assertNotContains(response,'NOTA-AJENA')
        self.client.post('/foro/', {'title':'<script>alert(1)</script>', 'body':'<img src=x onerror=alert(1)>'})
        topic=Topic.objects.get(author=self.first)
        response=self.client.get(f'/foro/tema/{topic.pk}/')
        self.assertNotContains(response,'<script>alert')
        self.assertContains(response,'&lt;script&gt;')
        self.assertIn('Content-Security-Policy',response)

    def test_screenshots_and_moderation(self):
        self.client.force_login(self.first)
        buffer=io.BytesIO(); Image.new('RGB',(20,20),'red').save(buffer,format='PNG')
        image=SimpleUploadedFile('../../dato.png',buffer.getvalue(),content_type='image/png')
        self.assertEqual(self.client.post('/foro/', {'title':'Error','body':'Detalle','screenshot':image}).status_code,302)
        topic=Topic.objects.get()
        self.assertTrue(bytes(topic.image).startswith(b'\x89PNG'))
        self.assertEqual(self.client.get(f'/foro/captura/{topic.pk}/').status_code,200)
        self.client.logout()
        self.assertEqual(self.client.get(f'/foro/captura/{topic.pk}/').status_code,302)
        self.client.force_login(self.second)
        topic.visible=False; topic.save()
        self.assertEqual(self.client.get(f'/foro/captura/{topic.pk}/').status_code,404)
        bad=SimpleUploadedFile('falsa.png',b'<svg onload="alert(1)"/>',content_type='image/png')
        form=TopicForm({'title':'Prueba','body':'Prueba'},{'screenshot':bad})
        self.assertFalse(form.is_valid())

    def test_method_and_no_source_exposure(self):
        self.client.force_login(self.first)
        self.assertEqual(self.client.get('/foro/salir/').status_code,405)
        for route in ['/settings.py','/data/sitio.sqlite3','/../src/main.st','/.env']:
            self.assertEqual(self.client.get(route).status_code,404)

    def test_only_author_or_staff_can_remove(self):
        topic=Topic.objects.create(author=self.first,title='Caso',body='Detalle',image=b'imagen')
        self.client.force_login(self.second)
        self.assertEqual(self.client.post(f'/foro/retirar/{topic.pk}/').status_code,403)
        self.client.force_login(self.first)
        self.assertEqual(self.client.get(f'/foro/retirar/{topic.pk}/').status_code,405)
        self.assertEqual(self.client.post(f'/foro/retirar/{topic.pk}/').status_code,302)
        topic.refresh_from_db()
        self.assertFalse(topic.visible)
        self.assertFalse(topic.image)
