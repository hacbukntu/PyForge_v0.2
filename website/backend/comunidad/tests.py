"""Regresiones del foro público de solo lectura y de recursos protegidos."""
import json
from django.contrib.auth import get_user_model
from django.test import TestCase
from pathlib import Path
from .models import Reply, Topic


class ForumTests(TestCase):
    """Usa datos temporales: nada se crea en la base de datos real."""
    @classmethod
    def setUpTestData(cls):
        author = get_user_model().objects.create_user('ana', password='Prueba-larga!1357')
        cls.topic = Topic.objects.create(
            author=author,
            title='<script>alert(1)</script>',
            body='Contenido público',
            image=b'\x89PNG\r\n\x1a\n',
        )
        Reply.objects.create(topic=cls.topic, author=author, body='Respuesta pública')

    def test_forum_and_topics_are_public_and_read_only(self):
        response = self.client.get('/foro/')
        self.assertEqual(response.status_code, 200)
        self.assertContains(response, '&lt;script&gt;')
        self.assertNotContains(response, '<form')
        self.assertNotContains(response, '/foro/entrar/')
        self.assertNotContains(response, '/foro/registro/')

        response = self.client.get(f'/foro/tema/{self.topic.pk}/')
        self.assertEqual(response.status_code, 200)
        self.assertContains(response, 'Respuesta pública')
        self.assertNotContains(response, '<form')
        self.assertEqual(
            self.client.get(f'/foro/captura/{self.topic.pk}/').status_code,
            200,
        )

        self.assertEqual(self.client.post('/foro/').status_code, 405)
        self.assertEqual(self.client.post(f'/foro/tema/{self.topic.pk}/').status_code, 405)
        self.assertEqual(self.client.post(f'/foro/captura/{self.topic.pk}/').status_code, 405)

    def test_public_auth_and_write_routes_are_removed(self):
        for route in (
            '/foro/entrar/',
            '/foro/registro/',
            '/foro/salir/',
            f'/foro/retirar/{self.topic.pk}/',
            '/foro/notas/',
            '/foro/st/',
        ):
            with self.subTest(route=route):
                self.assertEqual(self.client.get(route).status_code, 404)

    def test_download_page_and_published_artifacts_are_served(self):
        response = self.client.get('/descargas.html')
        self.assertEqual(response.status_code, 200)
        self.assertContains(response, 'Elige tu plataforma')

        catalog = Path(__file__).resolve().parents[3] / 'dist' / 'site' / 'curso.json'
        downloads = json.loads(catalog.read_text())['downloads']
        self.assertTrue(downloads)
        for item in downloads:
            with self.subTest(artifact=item['name']):
                self.assertEqual(self.client.head('/' + item['url']).status_code, 200)

    def test_hidden_topics_and_private_files_stay_unavailable(self):
        self.topic.visible = False
        self.topic.save(update_fields=['visible'])
        self.assertEqual(self.client.get(f'/foro/tema/{self.topic.pk}/').status_code, 404)
        self.assertEqual(self.client.get(f'/foro/captura/{self.topic.pk}/').status_code, 404)
        for route in ('/settings.py', '/data/sitio.sqlite3', '/../src/main.st', '/.env'):
            with self.subTest(route=route):
                self.assertEqual(self.client.get(route).status_code, 404)
