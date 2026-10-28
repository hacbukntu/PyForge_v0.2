"""Rutas explícitas; nunca se sirven fuentes, bases SQL ni carpetas del proyecto."""
from django.contrib import admin
from django.contrib.auth.views import LogoutView
from django.shortcuts import redirect
from django.urls import path
from comunidad import views
urlpatterns = [path('', lambda request: redirect('topics')), path('foro/',views.topics,name='topics'), path('foro/entrar/',views.SignIn.as_view(),name='login'), path('foro/registro/',views.register,name='register'), path('foro/salir/',LogoutView.as_view(),name='logout'), path('foro/tema/<int:pk>/',views.topic,name='topic'), path('foro/retirar/<int:pk>/',views.remove_topic,name='remove_topic'), path('foro/captura/<int:pk>/',views.screenshot,name='screenshot'), path('notas/',views.notes,name='notes'), path('vista-st/',views.st_view,name='st'), path('administracion/',admin.site.urls)]
