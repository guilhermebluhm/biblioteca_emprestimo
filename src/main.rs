use std::{collections::HashMap};

#[derive(Debug, PartialEq, Eq)]
enum Genre{
    Fiction,
    NonFiction,
    Scifi,
    Mystery,
    Documentary,
    Tech
}

#[derive(Debug)]
struct Book{
    id: u32,
    title: String,
    genre: Genre,
    avaliable: bool
}

#[derive(Debug)]
struct Dvd{
    id: u32,
    title: String,
    director: String,
    genre: Genre,
    avaliable: bool
}

#[derive(Debug)]
struct Magazine{
    id: u32,
    title: String,
    edition: String,
    avaliable: bool
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Item_kind{
    Book,
    Dvd,
    Magazine,
    None
}

#[derive(Debug)]
struct User{
    id: u32,
    name: String,
    active_loan_ids: Vec<Loan>,
    loan_history: Vec<Loan>,
    reservation_ids: Vec<Reservation>,
    max_loans: u32,
    unpaid_fine: f64
}

#[derive(Debug)]
struct Reservation{
    id: u32,
    item_id: u32,
    user_id: u32,
    reserved_on: u32,
    notified: bool
}

#[derive(Debug, Clone)]
struct Loan{
    id: usize,
    item_id: u32,
    item_kind: Item_kind,
    user_id: u32,
    checkout_today: u32,
    due_day: u32,
    returned: bool,
    returned_day: Option<u32>,
    renewels: u32
}

#[derive(Debug)]
struct Library{
    books: HashMap<u32, Book>,
    dvds: HashMap<u32, Dvd>,
    magazines: HashMap<u32, Magazine>,
    users: HashMap<u32, User>,
    loans_library: Vec<Loan>,
    reservation_library: Vec<Reservation>,
    next_loan_id: usize,
    next_reservation_id: usize,
    today: u32,
    fine_per_day: f64,
    fine_block_threshold: f64 
}

enum LoanError{
    LoanNotFound(usize),
    AlreadyReturned(usize), //O loan já foi devolvido (returned == true). Carrega o loan_id.
}

enum BusinessLogicErrorCheckout{
    ItemUnavailable, //O item existe mas available é false. Carrega item_id e title para mensagem clara.
    LimitExceeded, //O usuário já atingiu max_loans. Carrega user_name e max para a mensagem.
    FineBlocked(String, f64), //O unpaid_fine do usuário supera fine_block_threshold. Carrega user_name e o valor da multa.
    ReservedByOther, //Há uma reserva ativa para este item feita por um usuário diferente do solicitante. Carrega item_id e o nome do reservante. 
                    //O item não pode ser emprestado para outra pessoa enquanto houver reserva.
    UserNotFound(u32),
    ItemNotFound(u32),
}

enum BusinessLogicErrorReturnedRenew{
    MaxRenewalsReached, //loan.renewals já atingiu o valor de max_renewals() do item. Carrega loan_id e o limite máximo para a mensagem
    RenewalBlockedByReservation //Existe ao menos uma reserva pendente (não notificada) para o item deste loan. 
                                //A renovação é negada para liberar o item para quem está esperando. Carrega item_id
}

enum BusinessLogicErrorReserve{
    ItemAlreadyAvailable(u32), //O usuário tentou reservar um item que está disponível. O sistema deve orientá-lo a fazer o empréstimo diretamente. Carrega item_id.
    AlreadyReservedByUser(u32) //O mesmo usuário já tem uma reserva ativa para este item. Não é permitido reservar duas vezes o mesmo item. Carrega item_id.
}

impl Library{

    fn find_book(&self, id: u32) -> Option<&Book> {
        self.books.get(&id)
    }

    fn find_dvd(&self, id: u32) -> Option<&Dvd> {
        self.dvds.get(&id)
    }

    fn find_magazine(&self, id: u32) -> Option<&Magazine> {
        self.magazines.get(&id)
    }

    fn set_avaliable(&mut self, item_id: u32, itemKind: Item_kind, flag: bool) -> () {

        match itemKind {
            Item_kind::Book => {
                if let Some(b) = self.books.get_mut(&item_id){
                    b.avaliable = flag;
                }
            }
            Item_kind::Dvd => {
                if let Some(d) = self.dvds.get_mut(&item_id){
                    d.avaliable = flag;
                }
            }
            Item_kind::Magazine => {
                if let Some(m) = self.magazines.get_mut(&item_id){
                    m.avaliable = flag;
                }
            }
            _ => ()
        }

    }

    fn checkout(&mut self, user_id: u32, item_id: u32, itemKind: Item_kind) -> Result<usize, BusinessLogicErrorCheckout> {

        if let Some(x) = self.users.get(&user_id) {
            if x.unpaid_fine > self.fine_block_threshold {
                return Err(BusinessLogicErrorCheckout::FineBlocked(x.name.clone(), x.unpaid_fine))
            }
            if x.active_loan_ids.len() >= (x.max_loans as usize) {
                return Err(BusinessLogicErrorCheckout::LimitExceeded)
            }
        }
        else{
            return Err(BusinessLogicErrorCheckout::UserNotFound(user_id))
        }

        let user = self.users.get(&user_id).unwrap();
        if let Some(x) = user.active_loan_ids.iter().find(|u| u.item_id == item_id){
            if x.user_id != user_id {
                return Err(BusinessLogicErrorCheckout::ReservedByOther)
            }
        }
            
        let mut days_loan = 0;
        let mut actual_loan_id = 0;
        let mut due_day_calculate = 0;

        match itemKind {
            Item_kind::Book => {

                if let Some(x) = self.find_book(item_id){
                    if !x.avaliable{
                        return Err(BusinessLogicErrorCheckout::ItemUnavailable)
                    }
                    days_loan = x.days_loan_limit();
                }
                else{
                    return Err(BusinessLogicErrorCheckout::ItemNotFound(item_id))
                }

                actual_loan_id = self.next_loan_id;
                due_day_calculate = self.today + days_loan;
                let loan = Loan{id: self.next_loan_id+1, item_id, item_kind: Item_kind::Book, 
                    user_id, checkout_today: self.today, due_day: due_day_calculate, 
                    returned: false, returned_day: None, renewels: 0};

                let user_mut = self.users.get_mut(&user_id).unwrap();
                user_mut.active_loan_ids.push(loan.clone());
                user_mut.loan_history.push(loan);
                let book_mut = self.books.get_mut(&item_id).unwrap();
                book_mut.avaliable = false;

            }
            Item_kind::Dvd => {
                if let Some(x) = self.find_dvd(item_id){
                    if !x.avaliable{
                        return Err(BusinessLogicErrorCheckout::ItemUnavailable)
                    }
                    days_loan = x.days_loan_limit();
                }
                else{
                    return Err(BusinessLogicErrorCheckout::ItemNotFound(item_id))
                }

                actual_loan_id = self.next_loan_id;
                due_day_calculate = self.today + days_loan;
                let loan = Loan{id: self.next_loan_id+1, item_id, item_kind: Item_kind::Book, 
                    user_id, checkout_today: self.today, due_day: due_day_calculate, 
                    returned: false, returned_day: None, renewels: 0};

                let user_mut = self.users.get_mut(&user_id).unwrap();
                user_mut.active_loan_ids.push(loan.clone());
                user_mut.loan_history.push(loan);
                let dvd_mut = self.dvds.get_mut(&item_id).unwrap();
                dvd_mut.avaliable = false;

            }
            Item_kind::Magazine => {
                if let Some(x) = self.find_magazine(item_id){
                    if !x.avaliable{
                        return Err(BusinessLogicErrorCheckout::ItemUnavailable)
                    }
                    days_loan = x.days_loan_limit();
                }
                else{
                    return Err(BusinessLogicErrorCheckout::ItemNotFound(item_id))
                }

                actual_loan_id = self.next_loan_id;
                due_day_calculate = self.today + days_loan;
                let loan = Loan{id: self.next_loan_id+1, item_id, item_kind: Item_kind::Book, 
                    user_id, checkout_today: self.today, due_day: due_day_calculate, 
                    returned: false, returned_day: None, renewels: 0};

                let user_mut = self.users.get_mut(&user_id).unwrap();
                user_mut.active_loan_ids.push(loan.clone());
                user_mut.loan_history.push(loan);
                let magazine_mut = self.magazines.get_mut(&item_id).unwrap();
                magazine_mut.avaliable = false;

            }
            _ => ()
        }

        let user_reservations = self.users.get_mut(&user_id).unwrap();

        //removendo a entrada deste itemKind que estava previamente reservado
        user_reservations.reservation_ids.retain(|f| f.item_id != item_id);
        self.reservation_library.retain(|f| f.item_id != item_id);

        Ok(actual_loan_id+1)
    }

    fn return_item(&mut self, ID: usize) -> Result<f64, LoanError> {

        let mut due_day: u32 = 0;

        if let Some(x) = self.loans_library.iter().find(|f| f.id == ID){
            if x.returned {
                return Err(LoanError::AlreadyReturned(ID))
            }

            due_day = x.due_day;

        }
        else{
            return Err(LoanError::LoanNotFound(ID))
        }

        let mut diff: f64 = 0.00;
        let mut fine: f64 = 0.00;

        if self.today > due_day {
            diff = (self.today as f64) - (due_day as f64);
            fine = diff * self.fine_per_day;
        }

        let loan = self.loans_library.get_mut(ID).unwrap();
        loan.returned = true;
        loan.returned_day = Some(self.today);

        let user_loan = self.users.get_mut(&loan.user_id).unwrap();
        user_loan.unpaid_fine += fine;
        user_loan.active_loan_ids.retain(|f| f.item_id != user_loan.id);

        if let Some(x) = self.reservation_library.iter_mut()
        .min_by_key(|f| f.id == user_loan.id && f.notified == false){
            x.notified = true;
        }

        Ok(fine)
    }

}

trait LibraryItem{
    fn item_id(&self) -> u32;
    fn title(&self) -> &str;
    fn media_type(&self) -> &str;
    fn genre_label(&self) -> &str {
        "N/A"
    }
}

impl Genre {
    fn label(&self) -> &str{
        match self {
            Genre::Documentary => "Documentary",
            Genre::Fiction => "Fiction",
            Genre::Mystery => "Mystery",
            Genre::NonFiction => "Non-Fiction",
            Genre::Scifi => "Sci-Fi",
            Genre::Tech => "Tech"
        }
    }
}

impl LibraryItem for Book {
    fn item_id(&self) -> u32 {
        self.id
    }
    fn media_type(&self) -> &str {
        "BOOK"
    }
    fn title(&self) -> &str {
        &self.title
    }
    fn genre_label(&self) -> &str {
        self.genre.label()
    }
}

impl LibraryItem for Dvd{
    fn item_id(&self) -> u32 {
        self.id
    }
    fn media_type(&self) -> &str {
        "DVD"
    }
    fn title(&self) -> &str {
        &self.title
    }
    fn genre_label(&self) -> &str {
        self.genre.label()
    }
}

impl LibraryItem for Magazine{
    fn item_id(&self) -> u32 {
        self.id
    }
    fn media_type(&self) -> &str {
        "Magazine"
    }
    fn title(&self) -> &str {
        &self.title
    }
}

trait Loanable : LibraryItem {
    fn can_loan(&self) -> bool;
    fn days_loan_limit(&self) -> u32;
    fn max_renewals(&self) -> u32;
}

impl Loanable for Book{
    fn can_loan(&self) -> bool {
        self.avaliable
    }
    fn days_loan_limit(&self) -> u32 {
        14
    }
    fn max_renewals(&self) -> u32 {
        2
    }
}

impl Loanable for Dvd{
    fn can_loan(&self) -> bool {
        self.avaliable
    }
    fn days_loan_limit(&self) -> u32 {
        7
    }
    fn max_renewals(&self) -> u32 {
        1
    }
}

impl Loanable for Magazine{
    fn can_loan(&self) -> bool {
        self.avaliable
    }
    fn days_loan_limit(&self) -> u32 {
        3
    }
    fn max_renewals(&self) -> u32 {
        0
    }
}

fn main(){
    println!("");
}