provider tracing {
    probe trace(char *, char*, char *);
    probe debug(char *, char*, char *);
    probe info(char *, char*, char *);
    probe warn(char *, char*, char *);
    probe error(char *, char*, char *);
    probe event(char *, char *, char*);
    probe enter(char *, char *);
    probe exit(char *, char *);
};
