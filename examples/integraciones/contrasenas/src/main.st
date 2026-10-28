importar "cifrado";
// Contraseña ficticia de prueba. En una aplicación real se recibe de forma privada.
variable resumen = cifrado.hash_clave("solo-prueba-local");
mostrar(resumen);
variable clave = "solo-prueba-local";
mostrar(cifrado.verificar_clave(clave, resumen));
mostrar(cifrado.verificar_clave("incorrecta", resumen));
