/* ABI 1. UTF-8 terminado en NUL; el anfitrión garantiza vigencia de punteros.
 * Todas las cadenas devueltas se liberan con pyforge_django_free.
 * No destruir el contexto mientras haya llamadas o un servidor activos. */
#ifndef PYFORGE_DJANGO_H
#define PYFORGE_DJANGO_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct Django Django;
typedef char *(*PyForgeVista)(const char *peticion_json);
typedef void (*PyForgeLiberarVista)(char *respuesta_json);
uint32_t pyforge_django_abi_version(void);
Django *pyforge_django_new(const char *sqlite_url);
void pyforge_django_destroy(Django *contexto);
char *pyforge_django_call(const char *operacion_json);
char *pyforge_django_orm(Django *contexto, const char *operacion_json);
int32_t pyforge_django_route(Django *contexto, const char *ruta, const char *nombre,
                           PyForgeVista vista, PyForgeLiberarVista liberar);
int32_t pyforge_django_listen(Django *contexto, uint16_t puerto, const char *origen);
void pyforge_django_free(char *salida);
#ifdef __cplusplus
}
#endif
#endif
