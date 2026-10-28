// Excepciones explícitas que atraviesan funciones nativas.
funcion validar_edad(edad: entero) {
    si edad < 0 { lanzar "La edad no puede ser negativa"; }
}

intentar {
    validar_edad(-1);
} capturar error {
    mostrar("Aviso: " + error);
} finalmente {
    // Se ejecuta también al retornar, romper, continuar o propagar otro lanzar.
    mostrar("Validación terminada");
}
