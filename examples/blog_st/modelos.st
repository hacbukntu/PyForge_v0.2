// Validación de dominio ST. La persistencia del puente actual pertenece al ORM Django.
// La declaración modelo Usuario : django.Modelo todavía no existe en el parser AOT.
funcion titulo_valido(titulo: cadena) -> booleano {
    retornar longitud(titulo) > 0 y longitud(titulo) <= 120;
}
