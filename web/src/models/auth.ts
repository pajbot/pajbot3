export interface UserDetails {
    id: string,
    login: string,
    display_name: string,
    type: string,
    broadcaster_type: string,
    description: string,
    profile_image_url: string,
    offline_image_url: string,
    view_count: number,
    created_at: Date
}

export interface UserAuthorization {
    access_token: string;
    valid_until: Date;
    user_details: UserDetails;
}
